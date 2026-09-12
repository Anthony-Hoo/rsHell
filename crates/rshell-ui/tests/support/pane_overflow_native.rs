use super::fluent_native::wait_for_frame;
use super::*;

pub(super) fn run() {
    let mut view = native_workspace_fixture().view;
    let tab = &mut view.workspace.tabs[0];
    let ids = tab.pane_tree.pane_ids();
    let target = ids[2];
    let target_session = tab.pane_tree.session_id(target).unwrap().unwrap();
    tab.pane_tree = tab.pane_tree.clone().close(ids[1]).unwrap();
    let outputs = Rc::new(RefCell::new(Vec::new()));
    let recorded = outputs.clone();
    let host = PaneHost::builder()
        .launch(PaneHostInit {
            view_model: view.clone(),
            startup_probe: None,
        })
        .connect_receiver(move |_, output| recorded.borrow_mut().push(output));
    let window = gtk::Window::builder()
        .decorated(false)
        .default_width(280)
        .default_height(400)
        .child(host.widget())
        .build();
    window.present();
    wait_for_frame(&window, "narrow error pane", |root| {
        descendants(root)
            .iter()
            .any(|w| w.has_css_class("pane-error-actions") && w.is_mapped())
    });
    let error_region = descendants(host.widget())
        .into_iter()
        .find(|w| w.has_css_class("pane-error-actions"))
        .unwrap();
    for action in ["Retry", "Copy Diagnostics", "Edit Connection", "Close"] {
        click_overflow(&error_region, action);
        assert!(flush_gtk());
        match action {
            "Retry" => assert!(outputs.borrow().iter().any(|o| matches!(o, PaneHostOutput::Command(c) if matches!(c.as_ref(), UiCommand::RetryPane(p) if *p == target)))),
            "Close" => assert!(outputs.borrow().iter().any(|o| matches!(o, PaneHostOutput::Command(c) if matches!(c.as_ref(), UiCommand::ClosePane(p) if *p == target)))),
            "Edit Connection" => {
                let PaneLaunchTarget::Connection { id, .. } = view.pane_launches[&target] else { panic!("managed target") };
                assert!(outputs.borrow().iter().any(|o| matches!(o, PaneHostOutput::EditConnection(actual) if *actual == id)));
            }
            "Copy Diagnostics" => {
                let text = gtk::glib::MainContext::default().block_on(window.clipboard().read_text_future()).unwrap().unwrap();
                assert!(text.contains("Authentication") || text.contains("authentication"));
                assert!(!text.contains("Network"));
            }
            _ => unreachable!(),
        }
    }
    view.error_panes.remove(&target_session);
    view.session_states
        .insert(target_session, SessionState::Connected);
    host.emit(PaneHostMsg::SetViewModel(Box::new(view)));
    wait_for_frame(&window, "narrow connected panes", |root| {
        !descendants(root)
            .iter()
            .any(|w| w.has_css_class("pane-error-actions"))
    });
    let surfaces = descendants(host.widget())
        .into_iter()
        .filter(|w| w.has_css_class("pane-surface"))
        .collect::<Vec<_>>();
    let row = descendants(&surfaces[1])
        .into_iter()
        .find(|w| w.has_css_class("pane-command-row"))
        .unwrap();
    let narrow_height = row.compute_bounds(host.widget()).unwrap().height();
    for (name, axis) in [
        ("Split horizontally", SplitAxis::Horizontal),
        ("Split vertically", SplitAxis::Vertical),
    ] {
        click_overflow(&surfaces[1], name);
        assert!(flush_gtk());
        assert!(outputs.borrow().iter().any(|o| matches!(o, PaneHostOutput::Command(c) if matches!(c.as_ref(), UiCommand::Split { pane, axis: actual } if *pane == target && *actual == axis))));
    }
    window.set_default_size(1200, 400);
    wait_for_frame(&window, "wide actions allocated", |root| {
        root.width() > 1000
            && !descendants(root)
                .iter()
                .any(|w| w.is_mapped() && w.is::<gtk::MenuButton>())
    });
    let wide_height = row.compute_bounds(host.widget()).unwrap().height();
    assert_eq!(
        narrow_height, wide_height,
        "More and direct pane actions must not change chrome height or terminal geometry"
    );
    println!(
        "PANE_OVERFLOW_NATIVE_PASS split_h=true split_v=true retry=true close=true edit=true diagnostics=true target=right_nonactive"
    );
    window.close();
    assert!(flush_gtk());
}

fn click_overflow(root: &gtk::Widget, action: &str) {
    let more = descendants(root)
        .into_iter()
        .find_map(|w| w.downcast::<gtk::MenuButton>().ok())
        .expect("narrow pane More");
    let window = more.root().unwrap().downcast::<gtk::Window>().unwrap();
    more.popup();
    let popover = more.popover().unwrap();
    let observed_more = more.clone();
    let observed_popover = popover.clone();
    wait_for_frame(&window, "pane More lifecycle", move |_| {
        !observed_more.is_mapped() || observed_popover.is_mapped()
    });
    assert!(
        more.is_mapped(),
        "opening More must not replace its pane/trigger when focus enters"
    );
    assert!(popover.is_mapped(), "More popover must remain open");
    super::surface_capture::capture(
        &popover,
        &format!(
            "pane-more-{}",
            action.replace(' ', "-").to_ascii_lowercase()
        ),
    );
    let row = button_by_tooltip(&popover, action);
    assert!(row.is_mapped() && row.is_sensitive());
    let b = row.compute_bounds(&popover).unwrap();
    assert!(b.width() >= 36.0 && b.height() >= 36.0);
    row.emit_clicked();
}
