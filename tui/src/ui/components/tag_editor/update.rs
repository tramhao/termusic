use anyhow::anyhow;
use termusiclib::songtag::TrackDLMsg;

use crate::ui::Model;
use crate::ui::ids::{Id, IdTagEditor};
use crate::ui::msg::{TEMsg, TFMsg};

impl Model {
    pub fn update_tageditor(&mut self, msg: TEMsg) {
        match msg {
            TEMsg::Open(path) => {
                self.mount_tageditor(&path);
            }
            TEMsg::Close => {
                if let Some(s) = self.tageditor_song.clone() {
                    // TODO: this should be re-done and take actual track ids themself, or at least verified to use the same functions to result in the same id
                    self.library_reload_with_node_focus(Some(s.path_as_id_str().to_string()));
                }
                self.umount_tageditor();
            }

            TEMsg::CounterDeleteOk => {
                self.te_delete_lyric();
            }
            TEMsg::SelectLyricOk(index) => {
                if let Some(mut song) = self.tageditor_song.take() {
                    song.set_lyric_selected_index(index);
                    // the unwrap should also never happen as all components should be properly mounted
                    self.init_by_song(song).unwrap();
                }
            }
            TEMsg::Search => {
                self.te_songtag_search();
            }
            TEMsg::Download(index) => {
                if let Err(e) = self.te_songtag_download(index) {
                    self.mount_error_popup(e.context("download by songtag"));
                }
            }
            TEMsg::Embed(index) => {
                if let Err(e) = self.te_load_lyric_and_photo(index) {
                    self.mount_error_popup(e.context("log lyric and photo"));
                }
            }
            TEMsg::EmbedDone(song) => {
                self.te_load_lyric_and_photo_done(song);
            }
            TEMsg::EmbedErr(err) => {
                self.mount_error_popup(anyhow!(err));
            }
            TEMsg::Save => {
                if let Err(e) = self.te_rename_song_by_tag() {
                    self.mount_error_popup(e.context("rename song by tag"));
                }
            }
            TEMsg::Focus(msg) => self.update_tag_editor_focus(msg),

            TEMsg::SearchLyricResult(msg) => self.te_update_lyric_results(msg),
            TEMsg::TrackDownloadResult(msg) => self.te_update_download_msg(msg),
            TEMsg::TrackDownloadPreError(err) => {
                self.mount_error_popup(anyhow!(err));
            }
        }
    }

    fn update_tag_editor_focus(&mut self, msg: TFMsg) {
        match msg {
            TFMsg::TextareaLyricBlurDown | TFMsg::InputTitleBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::InputArtist))
                    .ok();
            }
            TFMsg::InputArtistBlurDown | TFMsg::InputAlbumBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::InputTitle))
                    .ok();
            }
            TFMsg::InputTitleBlurDown | TFMsg::InputGenreBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::InputAlbum))
                    .ok();
            }
            TFMsg::InputAlbumBlurDown | TFMsg::TableLyricOptionsBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::InputGenre))
                    .ok();
            }
            TFMsg::InputGenreBlurDown | TFMsg::SelectLyricBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::TableLyricOptions))
                    .ok();
            }
            TFMsg::TableLyricOptionsBlurDown | TFMsg::CounterDeleteBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::SelectLyric))
                    .ok();
            }
            TFMsg::SelectLyricBlurDown | TFMsg::TextareaLyricBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::CounterDelete))
                    .ok();
            }
            TFMsg::CounterDeleteBlurDown | TFMsg::InputArtistBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::TextareaLyric))
                    .ok();
            }
        }
    }

    /// Handle all cases for [`TrackDLMsg`].
    fn te_update_download_msg(&mut self, msg: TrackDLMsg) {
        match msg {
            TrackDLMsg::Start(url, title) => {
                self.download_tracker.increase_one(&*url);
                self.show_message_timeout_label_help(
                    self.download_tracker.message_download_start(&title),
                    None,
                    None,
                    None,
                );
            }
            TrackDLMsg::Success(url) => {
                self.download_tracker.decrease_one(&url);
                self.show_message_timeout_label_help(
                    self.download_tracker.message_download_complete(),
                    None,
                    None,
                    None,
                );

                if self.app.mounted(&Id::TagEditor(IdTagEditor::LabelHint)) {
                    self.umount_tageditor();
                }
            }
            TrackDLMsg::Completed(_url, file) => {
                if self.download_tracker.visible() {
                    return;
                }
                self.library_reload_with_node_focus(file);
            }
            TrackDLMsg::Err(url, title, error_message) => {
                self.download_tracker.decrease_one(&url);
                self.mount_error_popup(anyhow!("download failed: {error_message}"));
                self.show_message_timeout_label_help(
                    self.download_tracker
                        .message_download_error_response(&title),
                    None,
                    None,
                    None,
                );
            }
            TrackDLMsg::ErrEmbedData(_url, title) => {
                self.mount_error_popup(anyhow!("download ok but tag info is not complete."));
                self.show_message_timeout_label_help(
                    self.download_tracker
                        .message_download_error_embed_data(&title),
                    None,
                    None,
                    None,
                );
            }
        }
    }
}
