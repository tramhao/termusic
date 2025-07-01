use crate::ui::components::tag_editor::te_footer::TEFooter;
/**
 * MIT License
 *
 * termusic - Copyright (C) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use crate::ui::components::{
    LabelGeneric, TECounterDelete, TEInputAlbum, TEInputArtist, TEInputGenre, TEInputTitle,
    TESelectLyric, TETableLyricOptions, TETextareaLyric,
};
use crate::ui::model::Model;
use crate::ui::utils::{draw_area_in_absolute, draw_area_top_right_absolute};
use anyhow::Result;
use std::borrow::Cow;
use std::path::Path;
use termusiclib::ids::{Id, IdTagEditor};
use tuirealm::State;
use tuirealm::props::{Alignment, AttrValue, Attribute, PropPayload, PropValue, TextSpan};
use tuirealm::ratatui::layout::{Constraint, Layout};
use tuirealm::ratatui::widgets::Clear;

use super::TETrack;

impl Model {
    #[allow(clippy::too_many_lines)]
    pub fn view_tag_editor(&mut self) {
        self.terminal
            .raw_mut()
            .draw(|f| {
                let select_lyric_len =
                    match self.app.state(&Id::TagEditor(IdTagEditor::SelectLyric)) {
                        Ok(State::One(_)) => 3,
                        _ => 8,
                    };
                if self.app.mounted(&Id::TagEditor(IdTagEditor::LabelHint)) {
                    f.render_widget(Clear, f.area());
                    let chunks_main = Layout::vertical([
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        // Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(1),
                    ])
                    .split(f.area());

                    let chunks_row1 =
                        Layout::horizontal([Constraint::Ratio(2, 5), Constraint::Ratio(3, 5)])
                            .split(chunks_main[1]);
                    let chunks_row2 = Layout::horizontal([
                        Constraint::Ratio(1, 4),
                        Constraint::Ratio(1, 4),
                        Constraint::Ratio(1, 4),
                        Constraint::Ratio(1, 4),
                    ])
                    .split(chunks_main[2]);
                    let chunks_row4 =
                        Layout::horizontal([Constraint::Ratio(3, 5), Constraint::Ratio(2, 5)])
                            .split(chunks_main[3]);

                    let chunks_row4_right = Layout::vertical([
                        Constraint::Length(select_lyric_len),
                        Constraint::Min(2),
                    ])
                    .split(chunks_row4[1]);

                    let chunks_row4_right_top =
                        Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                            .split(chunks_row4_right[0]);

                    // -- footer
                    if self.download_tracker.visible() {
                        let chunks_footer = Layout::horizontal([
                            Constraint::Length(1),
                            Constraint::Length(1),
                            Constraint::Min(10),
                        ])
                        .split(chunks_main[4]);

                        self.app.view(&Id::DownloadSpinner, f, chunks_footer[1]);
                        self.app.view(&Id::Label, f, chunks_footer[2]);
                    } else {
                        self.app.view(&Id::Label, f, chunks_main[4]);
                    }

                    self.app
                        .view(&Id::TagEditor(IdTagEditor::LabelHint), f, chunks_main[0]);
                    // self.app.view(&Id::Label, f, chunks_main[4]);
                    self.app
                        .view(&Id::TagEditor(IdTagEditor::InputArtist), f, chunks_row1[0]);
                    self.app
                        .view(&Id::TagEditor(IdTagEditor::InputTitle), f, chunks_row1[1]);
                    self.app
                        .view(&Id::TagEditor(IdTagEditor::InputAlbum), f, chunks_row2[0]);
                    self.app
                        .view(&Id::TagEditor(IdTagEditor::InputGenre), f, chunks_row2[1]);
                    self.app.view(
                        &Id::TagEditor(IdTagEditor::TableLyricOptions),
                        f,
                        chunks_row4[0],
                    );
                    self.app.view(
                        &Id::TagEditor(IdTagEditor::SelectLyric),
                        f,
                        chunks_row4_right_top[0],
                    );
                    self.app.view(
                        &Id::TagEditor(IdTagEditor::CounterDelete),
                        f,
                        chunks_row4_right_top[1],
                    );
                    self.app.view(
                        &Id::TagEditor(IdTagEditor::TextareaLyric),
                        f,
                        chunks_row4_right[1],
                    );

                    if self.app.mounted(&Id::MessagePopup) {
                        let popup = draw_area_top_right_absolute(f.area(), 25, 4);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::MessagePopup, f, popup);
                    }
                    if self.app.mounted(&Id::ErrorPopup) {
                        let popup = draw_area_in_absolute(f.area(), 50, 4);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::ErrorPopup, f, popup);
                    }
                }
            })
            .expect("Expected to draw without error");
    }

    /// Mount / Remount the Tag Editor
    fn remount_tageditor(&mut self) -> Result<()> {
        self.app.remount(
            Id::Label,
            Box::new(TEFooter::new(&self.config_tui.read())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::TagEditor(IdTagEditor::LabelHint),
            Box::new(LabelGeneric::new(
                &self.config_tui.read(),
                "Press <ENTER> to search:",
            )),
            Vec::new(),
        )?;
        self.app.remount(
            Id::TagEditor(IdTagEditor::InputArtist),
            Box::new(TEInputArtist::new(self.config_tui.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::TagEditor(IdTagEditor::InputTitle),
            Box::new(TEInputTitle::new(self.config_tui.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::TagEditor(IdTagEditor::InputAlbum),
            Box::new(TEInputAlbum::new(self.config_tui.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::TagEditor(IdTagEditor::InputGenre),
            Box::new(TEInputGenre::new(self.config_tui.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::TagEditor(IdTagEditor::TableLyricOptions),
            Box::new(TETableLyricOptions::new(self.config_tui.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::TagEditor(IdTagEditor::SelectLyric),
            Box::new(TESelectLyric::new(self.config_tui.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::TagEditor(IdTagEditor::CounterDelete),
            Box::new(TECounterDelete::new(None, self.config_tui.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::TagEditor(IdTagEditor::TextareaLyric),
            Box::new(TETextareaLyric::new(self.config_tui.clone())),
            Vec::new(),
        )?;

        Ok(())
    }

    /// Mount the tageditor with the selected node id as the path.
    pub fn mount_tageditor(&mut self, node_id: &str) {
        let node_path: &Path = Path::new(node_id);
        if node_path.is_dir() {
            self.mount_error_popup(anyhow::anyhow!("{node_path:?} directory doesn't have tag!"));
            return;
        }

        let te_track = match TETrack::read_metadata_from_file(node_path) {
            Ok(v) => v,
            Err(err) => {
                self.mount_error_popup(err.context(node_path.display().to_string()));
                return;
            }
        };

        self.remount_tageditor().unwrap();

        self.app
            .active(&Id::TagEditor(IdTagEditor::InputArtist))
            .ok();

        // the unwrap should also never happen as all components should be properly mounted
        self.init_by_song(te_track).unwrap();

        if let Err(err) = self.update_photo() {
            self.mount_error_popup(err.context("update_photo"));
        }
    }

    /// Unmount the Tag Editor
    fn umount_tageditor_inner(&mut self) -> Result<()> {
        self.app.umount(&Id::TagEditor(IdTagEditor::LabelHint))?;
        self.app.umount(&Id::TagEditor(IdTagEditor::InputArtist))?;
        self.app.umount(&Id::TagEditor(IdTagEditor::InputTitle))?;
        self.app.umount(&Id::TagEditor(IdTagEditor::InputAlbum))?;
        self.app.umount(&Id::TagEditor(IdTagEditor::InputGenre))?;
        self.app
            .umount(&Id::TagEditor(IdTagEditor::TableLyricOptions))?;
        self.app.umount(&Id::TagEditor(IdTagEditor::SelectLyric))?;
        self.app
            .umount(&Id::TagEditor(IdTagEditor::CounterDelete))?;
        self.app
            .umount(&Id::TagEditor(IdTagEditor::TextareaLyric))?;

        Ok(())
    }

    pub fn umount_tageditor(&mut self) {
        self.mount_label_help();
        self.umount_tageditor_inner().unwrap();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(e.context("update_photo"));
        }
    }

    /// Set the Lyric section of the tag-editor to the Lyrics based on the provided Track
    #[allow(clippy::too_many_lines)]
    pub fn init_by_song(&mut self, s: TETrack) -> Result<()> {
        self.tageditor_song = Some(s);
        // Unwrap safe as we literally just assigned it
        let s = self.tageditor_song.as_ref().unwrap();
        if let Some(artist) = s.artist() {
            self.app.attr(
                &Id::TagEditor(IdTagEditor::InputArtist),
                Attribute::Value,
                AttrValue::String(artist.to_string()),
            )?;
        }

        if let Some(title) = s.title() {
            self.app.attr(
                &Id::TagEditor(IdTagEditor::InputTitle),
                Attribute::Value,
                AttrValue::String(title.to_string()),
            )?;
        }

        if let Some(album) = s.album() {
            self.app.attr(
                &Id::TagEditor(IdTagEditor::InputAlbum),
                Attribute::Value,
                AttrValue::String(album.to_string()),
            )?;
        }

        if let Some(genre) = s.genre() {
            self.app.attr(
                &Id::TagEditor(IdTagEditor::InputGenre),
                Attribute::Value,
                AttrValue::String(genre.to_string()),
            )?;
        }

        let lyric_frames = s.lyric_frames();

        if lyric_frames.is_empty() {
            self.init_by_song_no_lyric();
            return Ok(());
        }

        let vec_lang: Vec<PropValue> = lyric_frames
            .iter()
            .enumerate()
            .map(|(idx, v)| {
                let val = &v.description;
                // match idx with Delete counter
                let idx = idx + 1;
                if val.is_empty() {
                    format!("{idx} - {}", v.lang)
                } else {
                    format!("{idx} - {val}")
                }
            })
            .map(PropValue::Str)
            .collect();

        let selected_index = s.lyric_selected_index();
        // get access to "Lyric" instance for text and modified description for display
        let (Some(vec_lang_selected), Some(selected_desc)) =
            (s.lyric_selected(), vec_lang.get(selected_index))
        else {
            // this should not happen as if it is Some above, there should be at least one entry in the vec.
            self.init_by_song_no_lyric();
            return Ok(());
        };
        let selected_desc = selected_desc
            .as_str()
            .map_or_else(|| "No Description".into(), Cow::from);
        let selected_description = format!("Lyrics for {selected_desc}");
        let selected_index_display = selected_index + 1;

        let vec_lyric = vec_lang_selected
            .text
            .lines()
            .map(|line| PropValue::TextSpan(TextSpan::from(line.trim())))
            .collect();

        self.app.attr(
            &Id::TagEditor(IdTagEditor::SelectLyric),
            Attribute::Content,
            AttrValue::Payload(PropPayload::Vec(vec_lang)),
        )?;
        self.app.attr(
            &Id::TagEditor(IdTagEditor::CounterDelete),
            Attribute::Value,
            AttrValue::Payload(PropPayload::One(PropValue::Usize(selected_index_display))),
        )?;
        self.app.attr(
            &Id::TagEditor(IdTagEditor::TextareaLyric),
            Attribute::Title,
            AttrValue::Title((selected_description, Alignment::Left)),
        )?;

        self.app.attr(
            &Id::TagEditor(IdTagEditor::TextareaLyric),
            Attribute::Text,
            AttrValue::Payload(PropPayload::Vec(vec_lyric)),
        )?;

        Ok(())
    }

    /// Set the Lyric section of the tag-editor to "No Lyrics" (ie clear state)
    fn init_by_song_no_lyric(&mut self) {
        assert!(
            self.app
                .attr(
                    &Id::TagEditor(IdTagEditor::SelectLyric),
                    Attribute::Content,
                    AttrValue::Payload(PropPayload::Vec(
                        ["Empty"]
                            .iter()
                            .map(|x| PropValue::Str((*x).to_string()))
                            .collect(),
                    )),
                )
                .is_ok()
        );
        assert!(
            self.app
                .attr(
                    &Id::TagEditor(IdTagEditor::CounterDelete),
                    Attribute::Value,
                    AttrValue::Payload(PropPayload::None),
                )
                .is_ok()
        );

        assert!(
            self.app
                .attr(
                    &Id::TagEditor(IdTagEditor::TextareaLyric),
                    Attribute::Title,
                    AttrValue::Title(("Empty Lyrics".to_string(), Alignment::Left))
                )
                .is_ok()
        );
        assert!(
            self.app
                .attr(
                    &Id::TagEditor(IdTagEditor::TextareaLyric),
                    Attribute::Text,
                    AttrValue::Payload(PropPayload::Vec(vec![PropValue::TextSpan(
                        TextSpan::from("No Lyrics.")
                    ),]))
                )
                .is_ok()
        );
    }
}
