//! Module focusing on the way the transactions are orchestrated between worker threads

use std::{
    collections::HashMap,
    sync::mpsc::{SyncSender, sync_channel},
    thread::{Scope, ScopedJoinHandle},
};

use crate::{
    Error, TransactionRecord,
    domain::{AccountState, ClientId, Transaction},
    engine::{Accounts, logic::handle_transaction},
};

///
/// Processes an iterator of transactions and outputs the final state of client accounts, once the iterator is empty.
/// Uses a single thread.
///
pub(crate) fn process_transactions(
    transactions: impl IntoIterator<Item = Result<Transaction, Error>>,
    mut on_error: impl FnMut(Error),
    mut on_success: impl FnMut(TransactionRecord),
) -> HashMap<ClientId, AccountState> {
    let mut accounts = Accounts::default();

    for result in transactions {
        let tx = match result {
            Ok(tx) => tx,
            Err(err) => {
                on_error(err);
                continue;
            }
        };

        match handle_transaction(&tx, &mut accounts) {
            Ok(()) => on_success(TransactionRecord::from_domain(&tx)),
            Err(err) => on_error(err),
        }
    }

    accounts
}

///
/// Processes an iterator of transactions and outputs the final state of client accounts, once the iterator is empty.
/// Uses a number of worker threads provided by the `num_workers` argument, sharding the transactions between the worker
/// threads based on their `client_id`.
///
pub(crate) fn process_transactions_parallel(
    transactions: impl IntoIterator<Item = Result<Transaction, Error>>,
    on_error: impl FnMut(Error) + Send,
    on_success: impl FnMut(TransactionRecord) + Send,
    num_workers: usize,
    channel_capacity: usize,
) -> HashMap<ClientId, AccountState> {
    std::thread::scope(|s| {
        let (success_tx, error_tx) =
            spawn_callback_handlers(s, on_error, on_success, channel_capacity);

        let (worker_senders, worker_handles) = spawn_worker_threads(
            s,
            success_tx.clone(),
            error_tx.clone(),
            num_workers,
            channel_capacity,
        );

        // Main thread keeps a clone for parse errors
        let main_error_tx = error_tx.clone();

        // Drop originals — workers/callback threads hold their own clones
        drop(success_tx);
        drop(error_tx);

        // --- Main thread: parse and dispatch ---
        for result in transactions {
            match result {
                Ok(tx) => {
                    let client: u16 = tx.client_id().into();

                    // Sharding transactions based on the client id -> all transactions of the same client sent to the same worker
                    let worker_idx = client as usize % num_workers;

                    // Send fails only if the receiver was dropped (worker panicked);
                    // the join() below will surface that panic.
                    let _ = worker_senders[worker_idx].send(tx);
                }
                Err(e) => {
                    // Send fails only if the callback thread panicked; surfaced at join().
                    let _ = main_error_tx.send(e);
                }
            }
        }

        // Signal EOF: drop all senders
        drop(worker_senders);
        drop(main_error_tx);
        // → workers drain and exit → drop their success_tx/error_tx clones
        // → callback channels close → callback threads exit

        // --- Collect worker results ---
        let mut all_accounts = HashMap::new();
        for handle in worker_handles {
            let partition = handle.join().expect("worker thread does not panic");
            all_accounts.extend(partition);
        }

        all_accounts
    })
}

fn spawn_callback_handlers<'s, 'e>(
    s: &'s Scope<'s, 'e>,
    mut on_error: impl FnMut(Error) + Send + 's,
    mut on_success: impl FnMut(TransactionRecord) + Send + 's,
    channel_capacity: usize,
) -> (SyncSender<TransactionRecord>, SyncSender<Error>) {
    let (success_tx, success_rx) = sync_channel::<TransactionRecord>(channel_capacity);
    let (error_tx, error_rx) = sync_channel::<Error>(channel_capacity);

    s.spawn(move || {
        // worker processing the successful transactions
        for record in success_rx {
            on_success(record)
        }
    });

    s.spawn(move || {
        // worker processing the erroneous transactions
        for err in error_rx {
            on_error(err)
        }
    });

    (success_tx, error_tx)
}

fn spawn_worker_threads<'s, 'e>(
    s: &'s Scope<'s, 'e>,
    success_tx: SyncSender<TransactionRecord>,
    error_tx: SyncSender<Error>,
    num_workers: usize,
    channel_capacity: usize,
) -> (
    Vec<SyncSender<Transaction>>,
    Vec<ScopedJoinHandle<'s, Accounts>>,
) {
    let mut worker_senders = Vec::with_capacity(num_workers);
    let mut worker_handles = Vec::with_capacity(num_workers);

    for _ in 0..num_workers {
        let (tx_in, tx_out) = sync_channel::<Transaction>(channel_capacity);
        let stx = success_tx.clone();
        let etx = error_tx.clone();

        let handle = s.spawn(move || {
            let mut accounts = Accounts::default();
            for tx in tx_out {
                match handle_transaction(&tx, &mut accounts) {
                    Ok(()) => {
                        // Send fails only if the callback thread panicked;
                        // the caller's join() on worker handles will surface it.
                        let _ = stx.send(TransactionRecord::from_domain(&tx));
                    }
                    Err(e) => {
                        let _ = etx.send(e);
                    }
                }
            }
            accounts
        });

        worker_senders.push(tx_in);
        worker_handles.push(handle);
    }

    (worker_senders, worker_handles)
}
