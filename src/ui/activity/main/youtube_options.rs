use super::MainActivity;
use crate::ui::components::scrolltable;
use humantime::format_duration;
use std::time::Duration;
use tuirealm::props::{TableBuilder, TextSpan, TextSpanBuilder};
use tuirealm::PropsBuilder;
use unicode_truncate::{Alignment, UnicodeTruncateStr};

impl MainActivity {
    pub fn sync_youtube_options(&mut self) {
        if self.youtube_options.is_empty() {
            return;
        }
        let mut table: TableBuilder = TableBuilder::default();
        for (idx, record) in self.youtube_options.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }
            let duration = record.length_seconds;
            let duration_string = format!("{}", format_duration(Duration::from_secs(duration)));
            let duration_truncated = duration_string.unicode_pad(6, Alignment::Left, true);

            let title = record.title.clone();

            table
                .add_col(
                    TextSpanBuilder::new(format!("[{}] ", duration_truncated,).as_str()).build(),
                )
                .add_col(TextSpan::from(" "))
                .add_col(TextSpanBuilder::new(title.as_ref()).bold().build());
        }
        let table = table.build();

        match self.view.get_props(super::COMPONENT_SCROLLTABLE_YOUTUBE) {
            None => None,
            Some(props) => {
                let props = scrolltable::ScrollTablePropsBuilder::from(props.clone())
                    .with_table(Some(props.texts.title.unwrap()), table)
                    .build();
                self.view
                    .update(super::COMPONENT_SCROLLTABLE_YOUTUBE, props)
            }
        };
    }
}
