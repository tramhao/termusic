use std::{collections::HashSet, sync::Arc};

use parking_lot::RwLock;

/// A way to keep track of what downloads are currently happening
///
/// As long as items is above 0, a Download-spinner should play
#[derive(Debug, Clone)]
pub struct DownloadTracker {
    items: Arc<RwLock<HashSet<String>>>,
}

impl Default for DownloadTracker {
    fn default() -> Self {
        let items = HashSet::new();
        let items = Arc::new(RwLock::new(items));
        Self { items }
    }
}

#[allow(dead_code)]
impl DownloadTracker {
    /// Add a new entry with key `url`
    pub fn increase_one<U: Into<String>>(&self, url: U) {
        self.items.write().insert(url.into());
    }

    /// Remove a entry with key `url`
    pub fn decrease_one(&self, url: &str) {
        self.items.write().remove(url);
    }

    pub fn contains(&self, url: &str) -> bool {
        self.items.read().contains(url)
    }

    /// Should the download spinner be visible?
    pub fn visible(&self) -> bool {
        !self.items.read().is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.items.read().is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.read().len()
    }

    pub fn message_sync_success(&self) -> String {
        let len = self.len();
        if len > 0 {
            format!(
                " 1 of {} feeds was synced successfully! {len} are still running.",
                len + 1,
            )
        } else {
            " All feeds were synced successfully! ".to_string()
        }
    }
    pub fn message_feeds_added(&self) -> String {
        let len = self.len();
        if len > 0 {
            format!(
                " 1 of {} feeds was added successfully! {len} are still running.",
                len + 1,
            )
        } else {
            " All feeds were added successfully! ".to_string()
        }
    }

    pub fn message_feed_sync_failed(&self) -> String {
        let len = self.len();
        if len > 0 {
            format!(" 1 feed sync failed. {len} are still running. ",)
        } else {
            " 1 feed sync failed. ".to_string()
        }
    }

    pub fn message_sync_start(&self) -> String {
        let len = self.len();
        if len > 1 {
            format!(" {len} feeds are being fetching... ",)
        } else {
            " 1 feed is being fetching... ".to_string()
        }
    }

    pub fn message_download_start(&self, title: &str) -> String {
        let len = self.len();
        if len > 1 {
            format!(" {len} items downloading... ")
        } else {
            format!(" {len} item {title:^.20} downloading...",)
        }
    }

    pub fn message_download_complete(&self) -> String {
        let len = self.len();
        if len > 0 {
            format!(
                " 1 of {} Downloads Completed! {len} are still being processed.",
                len + 1,
            )
        } else {
            " All Downloads Successfully Completed! ".to_string()
        }
    }
    pub fn message_download_error_response(&self, title: &str) -> String {
        let len = self.len();
        if len > 0 {
            format!(" Failed to download item: {title:^.10}! No response from website. {len} downloads are still running. ",)
        } else {
            format!(" Failed to download item: {title:^.20}. No response from website.")
        }
    }
    pub fn message_download_error_file_create(&self, title: &str) -> String {
        let len = self.len();

        if len > 0 {
            format!(
                " Failed to download item: {title:^.10}! Unable to create a file. {len} downloads are still running. "
            )
        } else {
            format!(" Failed to download item: {title:^.20}. Unable to create a file.")
        }
    }

    pub fn message_download_error_file_write(&self, title: &str) -> String {
        let len = self.len();

        if len > 0 {
            format!(
                " Failed to download item: {title:^.10}! Cannot write to file. {len} downloads are still running. "
            )
        } else {
            format!(" Failed to download: {title:^.20}. Cannot write to file")
        }
    }

    pub fn message_download_error_embed_data(&self, title: &str) -> String {
        let len = self.len();

        if len > 0 {
            format!(
                " Failed to download item: {title:^.10}! Cannot embed data to file. {len} downloads are still running. "
            )
        } else {
            format!(" Failed to download: {title:^.20}. Cannot embed data to file.")
        }
    }
}
