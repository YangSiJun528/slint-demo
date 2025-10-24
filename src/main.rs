use std::rc::Rc;
use slint::{SharedString, VecModel, ModelRc};

slint::include_modules!();

fn main() {
    let ui = ColumnView::new().unwrap();

    // 테스트 데이터
    let test_data: Vec<ModelRc<SharedString>> = vec![
        Rc::new(VecModel::from(vec![
            "Documents".into(),
            "Downloads".into(),
            "Pictures".into(),
            "Music".into(),
        ])).into(),
        Rc::new(VecModel::from(vec![
            "Work".into(),
            "Personal".into(),
            "Archive".into(),
        ])).into(),
        Rc::new(VecModel::from(vec![
            "Project1".into(),
            "Project2".into(),
            "Project3".into(),
            "Project4".into(),
            "Project5".into(),
        ])).into(),
    ];

    ui.set_columns(Rc::new(VecModel::from(test_data)).into());

    ui.run().unwrap();
}
