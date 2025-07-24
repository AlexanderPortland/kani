// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use lazy_static::lazy_static;
use std::collections::BTreeMap;
use std::convert::identity;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpmc::{Receiver, Sender, channel};
use std::sync::mpsc::{SendError, TryRecvError};
use std::thread::JoinHandle;

use cbmc::irep::goto_binary_serde::write_goto_binary_file;
use cbmc::{InternedString, InternerSpecific, WithInterner};
use kani_metadata::ArtifactType;

use crate::codegen_cprover_gotoc::compiler_interface::write_file;

pub(crate) struct WorkUnit {
    pub symtab_goto: PathBuf,
    pub symbol_table: cbmc::goto_program::SymbolTable,
    pub vtable_restrictions: Option<kani_metadata::VtableCtxResults>,
    pub type_map: BTreeMap<InternedString, InternedString>,
    pub pretty_name_map: BTreeMap<InternedString, Option<InternedString>>,
    pub pretty: bool,
}

unsafe impl InternerSpecific for WorkUnit {}

impl WorkUnit {
    pub fn new(
        symtab_goto: &std::path::Path,
        symbol_table: &cbmc::goto_program::SymbolTable,
        vtable_restrictions: Option<kani_metadata::VtableCtxResults>,
        type_map: BTreeMap<InternedString, InternedString>,
        pretty_name_map: BTreeMap<InternedString, Option<InternedString>>,
        pretty: bool,
    ) -> Self {
        WorkUnit {
            symtab_goto: symtab_goto.to_path_buf(),
            symbol_table: symbol_table.clone(),
            vtable_restrictions,
            type_map,
            pretty_name_map,
            pretty,
        }
    }
}

// could also be thread local
const NUM_WORKERS: usize = 2;
lazy_static! {
    pub(crate) static ref WORKERS: Mutex<Option<Workers<NUM_WORKERS>>> = Mutex::new(None);
}

type WorkerReturn = ();

pub struct Workers<const N: usize> {
    pub(crate) work_queue: Sender<WithInterner<WorkUnit>>,
    join_handles: [JoinHandle<WorkerReturn>; N],
}

pub fn initialize_workers() {
    *WORKERS.lock().unwrap() = Some(Workers::new());
}

type WorkToSend = WithInterner<WorkUnit>;
pub fn send_work(work: WorkToSend) -> Result<(), Box<SendError<WorkToSend>>> {
    // box the err output because it's large and this will safe us memory use in the much more common success case
    WORKERS.lock().unwrap().as_ref().unwrap().work_queue.send(work).map_err(Box::new)
}

pub fn deinitialize_workers() -> Option<Workers<NUM_WORKERS>> {
    WORKERS.lock().unwrap().take()
}

impl<const N: usize> Workers<N> {
    pub fn new() -> Self {
        println!("i gotta be honest, im straight up worker threading all over the place...");
        let (work_queue_send, work_queue_recv) = channel();
        let join_handles = core::array::from_fn(identity).map(|_| {
            let new_work_queue_recv = work_queue_recv.clone();
            std::thread::spawn(move || {
                worker_loop(new_work_queue_recv);
            })
        });

        Workers { work_queue: work_queue_send, join_handles }
    }

    pub fn join_all(self) {
        // this structure itself maintains a reference to the work queue,
        // we have to close it so the channel will close and workers will know to exit
        drop(self.work_queue);

        for handle in self.join_handles {
            handle.join().unwrap();
        }
    }
}

fn worker_loop(work_queue: Receiver<WithInterner<WorkUnit>>) -> WorkerReturn {
    // println!("work loopin...");
    while let Ok(new_work) = work_queue.recv() {
        // this call to into_inner implicitly updates our thread local interner
        // println!("got work loopin...");
        handle_work(new_work.into_inner());
    }

    // Double check that the work queue has been closed by the sender.
    debug_assert!(matches!(work_queue.try_recv(), Err(TryRecvError::Disconnected)));
}

fn handle_work(
    WorkUnit { symtab_goto, symbol_table, vtable_restrictions, type_map, pretty_name_map, pretty }: WorkUnit,
) {
    write_file(&symtab_goto, ArtifactType::PrettyNameMap, &pretty_name_map, pretty);
    write_goto_binary_file(&symtab_goto, &symbol_table);
    write_file(&symtab_goto, ArtifactType::TypeMap, &type_map, pretty);
    // If they exist, write out vtable virtual call function pointer restrictions
    if let Some(restrictions) = vtable_restrictions {
        write_file(&symtab_goto, ArtifactType::VTableRestriction, &restrictions, pretty);
    }
}
