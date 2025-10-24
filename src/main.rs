use std::rc::Rc;
use slint::{VecModel, ModelRc};

slint::include_modules!();

fn main() {
    let ui = ColumnView::new().unwrap();

    let test_data: Vec<ModelRc<RawItem>> = vec![
        Rc::new(VecModel::from(vec![
            RawItem { item_type: ItemType::DIRECTORY, name: "Documents".into() },
            RawItem { item_type: ItemType::DIRECTORY, name: "Downloads".into() },
            RawItem { item_type: ItemType::DIRECTORY, name: "Downloads".into() },
            RawItem { item_type: ItemType::DIRECTORY, name: "Downloads".into() },
            RawItem { item_type: ItemType::DIRECTORY, name: "Downloads".into() },
            RawItem { item_type: ItemType::DIRECTORY, name: "Downloads".into() },
            RawItem { item_type: ItemType::FILE, name: "readme.txt".into() },
        ])).into(),
        Rc::new(VecModel::from(vec![
            RawItem { item_type: ItemType::DIRECTORY, name: "Work".into() },
            RawItem { item_type: ItemType::FILE, name: "report.pdf".into() },
        ])).into(),
    ];

    ui.set_column_data(Rc::new(VecModel::from(test_data)).into());
    ui.run().unwrap();
}
