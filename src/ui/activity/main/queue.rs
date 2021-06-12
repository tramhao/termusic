use super::MainActivity;
use super::COMPONENT_SCROLLTABLE;
use tuirealm::components::scrolltable;
use tuirealm::PropsBuilder;

use tuirealm::props::{TableBuilder, TextSpan};

impl MainActivity {
    pub fn add_queue(&mut self, item: String) {
        let line = String::from_utf8(item.into()).expect("utf8 error");
        self.queue_items.insert(0, line);

        self.sync_items();
    }

    fn sync_items(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.queue_items.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            table.add_col(TextSpan::from(String::from(record)));
        }
        let table = table.build();

        match self.view.get_props(COMPONENT_SCROLLTABLE) {
            None => None,
            Some(props) => {
                let props = scrolltable::ScrollTablePropsBuilder::from(props)
                    .with_table(Some(String::from("Queue")), table)
                    .build();
                self.view.update(COMPONENT_SCROLLTABLE, props)
            }
        };
    }
    pub fn delete_item(&mut self, index: usize) {
        self.queue_items.remove(index);
        self.sync_items();
    }
}
