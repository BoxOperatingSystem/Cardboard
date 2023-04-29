// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use core::ops::ControlFlow;

use amd64::paging::{pml4::PML4, PageTableEntry};
use tungstenkit::{
    syscall::{KernelMessage, Message},
    TerminationReason,
};

use crate::system::{
    proc::{scheduler::Scheduler, ThreadState},
    RegisterState,
};

pub fn send(
    scheduler: &mut Scheduler,
    state: &mut RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    if !scheduler.processes.contains_key(&state.rsi) {
        return ControlFlow::Break(Some(TerminationReason::NotFound));
    }

    let src = scheduler.current_pid.unwrap();
    let msg = Message::new(scheduler.msg_id_gen.next(), src, unsafe {
        core::slice::from_raw_parts(state.rdx as *const _, state.rcx as _)
    });
    scheduler.message_sources.insert(msg.id, src);

    let process = scheduler.processes.get_mut(&state.rsi).unwrap();
    process.track_msg(msg.id, state.rdx);
    process.messages.push_front(msg);
    unsafe {
        process.cr3.map_pages(
            state.rdx,
            state.rdx - tungstenkit::USER_PHYS_VIRT_OFFSET,
            (state.rcx + 0xFFF) / 0x1000,
            PageTableEntry::new().with_present(true).with_user(true),
        );
    }

    ControlFlow::Continue(())
}

pub fn receive(
    scheduler: &mut Scheduler,
    state: &mut RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    let process = scheduler.current_process_mut().unwrap();
    if let Some(msg) = process.messages.pop_back() {
        state.rax = msg.id;
        state.rdi = msg.pid;
        state.rsi = msg.data.as_ptr() as u64;
        state.rdx = msg.data.len() as u64;
    } else {
        scheduler.current_thread_mut().unwrap().state = ThreadState::Suspended;
    }

    ControlFlow::Continue(())
}

pub fn ack(
    scheduler: &mut Scheduler,
    state: &mut RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    let id = state.rsi;

    let Some(&src) = scheduler.message_sources.get(&id) else {
        return ControlFlow::Break(Some(TerminationReason::NotFound));
    };
    scheduler.message_sources.remove(&id);

    let process = scheduler.current_process_mut().unwrap();
    if src == 0 {
        let addr = process.message_allocations.get(&id).copied().unwrap();
        let (size, _) = process.allocations.get(&addr).copied().unwrap();
        let msg: KernelMessage = unsafe {
            postcard::from_bytes(core::slice::from_raw_parts(addr as *const _, size as _)).unwrap()
        };
        let KernelMessage::IRQFired(irq) = msg;
        crate::acpi::ioapic::set_irq_mask(irq, false);
    }
    process.free_msg(id);
    scheduler.msg_id_gen.free(id);

    ControlFlow::Continue(())
}
