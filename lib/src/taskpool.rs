use std::sync::Arc;

use futures::Future;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

/// Manages a taskpool of a given size of how many task to execute at once.
///
/// Also cancels all tasks spawned by this pool on [`Drop`]
#[must_use]
pub struct TaskPool {
    /// Semaphore to manage how many active tasks there at a time
    semaphore: Arc<Semaphore>,
    /// Cancel Token to stop a task on drop
    cancel_token: CancellationToken,
}

impl TaskPool {
    /// Creates a new [`TaskPool`] with a given amount of active tasks
    pub fn new(n_tasks: usize) -> TaskPool {
        let semaphore = Arc::new(Semaphore::new(n_tasks));
        let cancel_token = CancellationToken::new();

        TaskPool {
            semaphore,
            cancel_token,
        }
    }

    /// Adds a new task to the [`TaskPool`]
    ///
    /// see [`tokio::spawn`]
    ///
    /// Provided task will be cancelled on [`TaskPool`] [`Drop`]
    pub fn execute<F, T>(&self, func: F)
    where
        F: Future<Output = T> + Send + 'static,
        T: Send,
    {
        let semaphore = self.semaphore.clone();
        let token = self.cancel_token.clone();
        tokio::spawn(async move {
            // multiple "await" points, so combine them to a single future for the select
            let main = async {
                let Ok(_permit) = semaphore.acquire().await else {
                    // ignore / cancel task if semaphore is closed
                    // just for clarity, this "return" cancels the whole spawned task and does not execute "func.await"
                    return;
                };
                func.await;
            };

            tokio::select! {
                () = main => {},
                () = token.cancelled() => {}
            }
        });
    }
}

impl Drop for TaskPool {
    fn drop(&mut self) {
        // prevent new tasks from being added / executed
        self.semaphore.close();
        // cancel all tasks that were spawned with this token
        self.cancel_token.cancel();
    }
}
