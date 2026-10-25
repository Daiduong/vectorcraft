//! Panel bodies: what the dock area ([`crate::panel_docking`]) draws inside a docked, floating or
//! flyout panel.

use egui::Ui;

use crate::state::DockTab;
use crate::{VectorcraftApp, panels, widgets};

/// Panel `id`'s contents in a popped-out or floating panel `width` wide; the tabbed group's panels
/// fill (Layers) or scroll (Properties) `tall` points.
pub(crate) fn panel_body(app: &mut VectorcraftApp, ui: &mut Ui, id: &str, width: f32, tall: f32) {
    egui::Frame::NONE.inner_margin(egui::Margin::same(10)).show(ui, |ui| {
        ui.set_width(width - 20.0);
        match DockTab::from_id(id) {
            Some(DockTab::Properties) => {
                // At least `tall` when the sections are taller: the flyout's area keeps the size its
                // content needs, and egui sizes a new area at most 400 points high, so a scroll area
                // that only took the room it was given never grew past it.
                widgets::panel_scroll(
                    ui,
                    egui::ScrollArea::vertical().id_salt("props-flyout").max_height(tall).min_scrolled_height(tall).auto_shrink([false, true]),
                    |ui| panels::properties::show(app, ui),
                );
            }
            Some(DockTab::Layers) => {
                ui.set_max_height(tall);
                panels::layers::show(app, ui);
            }
            Some(DockTab::Libraries) => {
                // Its list scrolls inside `tall` (as in the dock) instead of growing the flyout.
                ui.set_max_height(tall);
                panels::libraries(app, ui);
            }
            None => panels::show_icon_panel(app, ui, id),
        }
    });
}

#[cfg(test)]
mod tests {
    use egui::vec2;
    use serde_json::json;
    use vectorcraft_engine::Session;

    use super::*;
    use crate::state::ICON_PANEL_GROUPS;
    use crate::theme;
    use crate::toolbar::tests::{wheel, widget_rects};

    /// The main column's width in the default layout.
    const DOCK_WIDTH: f32 = 310.0;

    fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
        crate::panel_docking::show(app, ui);
    }

    #[test]
    fn the_properties_scroll_bar_covers_no_control() {
        use egui::accesskit::{Action, Role};
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.run("file.new", json!({"width": 300, "height": 300})).unwrap();
        app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
        app.run("object.rasterize", json!({})).unwrap();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        theme::install_fonts(&ctx);
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1440.0, 600.0));
        let mut nodes = vec![];
        for k in 0..4 {
            let input = egui::RawInput { time: Some(f64::from(k) * 0.1), screen_rect: Some(screen), ..Default::default() };
            let mut out = ctx.run_ui(input, |ui| show(&mut app, ui));
            out.textures_delta.clear();
            nodes = out.platform_output.accesskit_update.unwrap().nodes;
        }
        let rect = |n: &egui::accesskit::Node| {
            n.bounds().map(|b| egui::Rect::from_min_max(egui::pos2(b.x0 as f32, b.y0 as f32), egui::pos2(b.x1 as f32, b.y1 as f32)))
        };
        // The Properties panel's is the right-most (the icon column's slim strip bar is left of it).
        let bar = nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::ScrollBar)
            .filter_map(|(_, n)| rect(n))
            .max_by(|a, b| a.left().total_cmp(&b.left()))
            .filter(|bar| bar.left() > screen.right() - DOCK_WIDTH);
        let bar = bar.expect("the Properties panel scrolls in a 600 pt window");
        let mut covered = vec![];
        for (_, n) in &nodes {
            let Some(r) = rect(n) else { continue };
            let control = n.role() != Role::ScrollBar && (n.supports_action(Action::Click) || n.supports_action(Action::Focus));
            if control && r.intersect(bar).is_positive() {
                covered.push(format!("{:?} {:?} at {r:?}", n.role(), n.label()));
            }
        }
        assert!(covered.is_empty(), "under the scroll bar at {bar:?}: {covered:#?}");
    }

    /// #3443: the Layers list's scroll bar sits beside the rows, not over the target and selection
    /// marks at their right edge.
    #[test]
    fn the_layers_scroll_bar_covers_no_row() {
        use egui::accesskit::Role;
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.run("file.new", json!({"width": 300, "height": 300})).unwrap();
        for k in 0..40 {
            app.run("shape.rectangle", json!({"x": k, "y": k, "width": 20, "height": 20})).unwrap();
        }
        app.run("window.panel", json!({"panel": "layers"})).unwrap();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        theme::install_fonts(&ctx);
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1440.0, 600.0));
        let mut nodes = vec![];
        for k in 0..4 {
            let input = egui::RawInput { time: Some(f64::from(k) * 0.1), screen_rect: Some(screen), ..Default::default() };
            let mut out = ctx.run_ui(input, |ui| show(&mut app, ui));
            out.textures_delta.clear();
            nodes = out.platform_output.accesskit_update.unwrap().nodes;
        }
        let rect = |n: &egui::accesskit::Node| {
            n.bounds().map(|b| egui::Rect::from_min_max(egui::pos2(b.x0 as f32, b.y0 as f32), egui::pos2(b.x1 as f32, b.y1 as f32)))
        };
        let bar = nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::ScrollBar)
            .filter_map(|(_, n)| rect(n))
            .max_by(|a, b| a.left().total_cmp(&b.left()))
            .filter(|bar| bar.left() > screen.right() - DOCK_WIDTH)
            .expect("40 layers scroll in a 600 pt window");
        let rows: Vec<egui::Rect> = nodes
            .iter()
            .filter(|(_, n)| n.role() != Role::ScrollBar)
            .filter_map(|(_, n)| rect(n))
            .filter(|r| r.top() >= bar.top() && r.bottom() <= bar.bottom() && r.right() > bar.left() - 40.0)
            .collect();
        assert!(!rows.is_empty(), "the rows beside the bar are in the tree");
        let covered: Vec<_> = rows.iter().filter(|r| r.intersect(bar).is_positive()).collect();
        assert!(covered.is_empty(), "under the scroll bar at {bar:?}: {covered:#?}");
    }

    const SCREEN: egui::Vec2 = vec2(1600.0, 1000.0);

    /// A frame of the dock and its flyout on a 1600 × 1000 window.
    struct Harness {
        app: VectorcraftApp,
        ctx: egui::Context,
        time: f64,
    }

    impl Harness {
        fn new() -> Self {
            let mut app = VectorcraftApp::new(Session::new(), Default::default());
            app.run("file.new", json!({"width": 300, "height": 300})).unwrap();
            app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
            let ctx = egui::Context::default();
            theme::install_fonts(&ctx);
            let mut h = Self { app, ctx, time: 0.0 };
            h.settle();
            h
        }

        fn frame(&mut self, events: Vec<egui::Event>) {
            self.time += 0.05;
            let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, SCREEN);
            let input = egui::RawInput { time: Some(self.time), events, screen_rect: Some(screen), ..Default::default() };
            let app = &mut self.app;
            self.ctx
                .run_ui(input, |ui| {
                    show(app, ui);
                    floating_panel(app, ui.ctx());
                    crate::dialogs::show(app, ui.ctx());
                })
                .textures_delta
                .clear();
        }

        fn settle(&mut self) {
            for _ in 0..3 {
                self.frame(vec![]);
            }
        }

        fn click(&mut self, at: egui::Pos2) {
            let button =
                |pressed| egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
            self.frame(vec![egui::Event::PointerMoved(at), button(true)]);
            self.frame(vec![button(false)]);
            self.settle();
        }

        /// The icon column's left edge as last drawn.
        fn column_left(&self) -> f32 {
            self.ctx.data(|d| d.get_temp::<f32>(column_left_id())).unwrap()
        }

        /// The panel icons of the column, top to bottom.
        fn icons(&self) -> Vec<Rect> {
            widget_rects(&self.ctx, vec2(30.0, 30.0))
        }

        /// The double arrow at the top right of the window (the dock's, or the collapsed column's).
        fn arrow(&self) -> egui::Pos2 {
            egui::pos2(SCREEN.x - 10.0, HEADER / 2.0)
        }
    }

    fn icon_panel_count() -> usize {
        ICON_PANEL_GROUPS.iter().map(|g| g.len()).sum()
    }

    #[test]
    fn the_double_arrow_collapses_the_dock_to_icons_and_back() {
        let mut h = Harness::new();
        assert!(!h.app.ui.dock_collapsed);
        // Expanded, the group (at least its 230-point minimum) is right of the column.
        let expanded = |left: f32| left <= SCREEN.x - ICON_COL - 230.0;
        assert!(expanded(h.column_left()), "expanded: the group is right of the column: {}", h.column_left());
        assert_eq!(h.icons().len(), icon_panel_count());
        h.click(h.arrow());
        assert!(h.app.ui.dock_collapsed, "the » collapses the dock");
        assert!(h.column_left() > SCREEN.x - ICON_COL - 2.0, "collapsed: only the icon column is left: {}", h.column_left());
        let icons = h.icons();
        assert_eq!(icons.len(), icon_panel_count() + 3, "Properties, Layers and Libraries became icons");
        assert!(icons[0].top() > HEADER, "the icons sit under the « strip");
        h.click(h.arrow());
        assert!(!h.app.ui.dock_collapsed, "the « expands the dock again");
        assert!(expanded(h.column_left()), "the group is back: {}", h.column_left());
        assert_eq!(h.icons().len(), icon_panel_count());
    }

    #[test]
    fn a_collapsed_panel_pops_out_left_of_the_column() {
        let mut h = Harness::new();
        h.app.run("window.collapseDock", json!({"collapsed": true})).unwrap();
        h.settle();
        // Properties, Layers, Libraries head the column.
        let layers = h.icons()[1];
        h.click(layers.center());
        assert_eq!(h.app.ui.open_panel.as_deref(), Some("layers"));
        h.settle();
        let rect = h.ctx.memory(|m| m.area_rect(egui::Id::new("icon-panel"))).unwrap();
        let column = h.column_left();
        assert!(rect.right() <= column, "the flyout {rect:?} stays left of the column at {column}");
        assert!(rect.left() >= 0.0 && rect.bottom() <= SCREEN.y, "the flyout {rect:?} is on screen");
        assert!(rect.height() > 200.0, "Layers gets room for its list: {rect:?}");
        // Its » puts it away.
        h.click(egui::pos2(rect.right() - 13.0, rect.top() + 13.0));
        assert_eq!(h.app.ui.open_panel, None, "the flyout's » closes it");
        // Expanding with a panel popped out shows its tab.
        h.click(layers.center());
        assert_eq!(h.app.ui.open_panel.as_deref(), Some("layers"));
        h.click(h.arrow());
        assert!(!h.app.ui.dock_collapsed);
        assert_eq!((h.app.ui.open_panel.as_deref(), h.app.ui.dock_tab), (None, DockTab::Layers));
        // Properties pops out as wide as the dock and shows its sections, not a sliver, also as
        // the first flyout of a session (a fresh harness: the flyouts share one area, which
        // would otherwise keep the size the Layers flyout gave it).
        let mut h = Harness::new();
        h.app.run("window.collapseDock", json!({"collapsed": true})).unwrap();
        h.app.run("window.panel", json!({"panel": "properties"})).unwrap();
        h.settle();
        let rect = h.ctx.memory(|m| m.area_rect(egui::Id::new("icon-panel"))).unwrap();
        assert!(rect.width() >= DOCK_WIDTH && rect.right() <= h.column_left(), "Properties flyout {rect:?}");
        // Taller than the 400 points egui sizes a new area at: its sections (Align, Quick
        // Actions) aren't cut off below Appearance.
        assert!(rect.height() > 450.0 && rect.bottom() <= SCREEN.y, "Properties flyout {rect:?}");
    }

    /// User Interface › Auto-Collapse Iconic Panels (#394): on, a click away from a popped-out
    /// panel puts it away; off (the default), the panel stays until its » or its icon.
    #[test]
    fn auto_collapse_puts_a_flyout_away_on_a_click_elsewhere() {
        let mut h = Harness::new();
        h.app.run("window.collapseDock", json!({"collapsed": true})).unwrap();
        h.settle();
        let (properties, layers) = (h.icons()[0], h.icons()[1]);
        let away = egui::pos2(300.0, 500.0);
        h.click(layers.center());
        assert_eq!(h.app.ui.open_panel.as_deref(), Some("layers"));
        h.click(away);
        assert_eq!(h.app.ui.open_panel.as_deref(), Some("layers"), "off: a click elsewhere leaves it");
        h.app.session.execute("prefs.set", &json!({"key": "autoCollapseIconPanels", "value": true})).unwrap();
        h.click(away);
        assert_eq!(h.app.ui.open_panel, None, "on: a click elsewhere puts it away");
        h.click(layers.center());
        assert_eq!(h.app.ui.open_panel.as_deref(), Some("layers"), "its icon still pops it out");
        let rect = h.ctx.memory(|m| m.area_rect(egui::Id::new("icon-panel"))).unwrap();
        h.click(egui::pos2(rect.center().x, rect.top() + 13.0 + 26.0 + 10.0));
        assert_eq!(h.app.ui.open_panel.as_deref(), Some("layers"), "a click inside keeps it");
        // The icons still swap the flyout and put it away.
        h.click(properties.center());
        assert_eq!(h.app.ui.open_panel.as_deref(), Some("properties"), "another icon swaps it");
        h.click(properties.center());
        assert_eq!(h.app.ui.open_panel, None, "the same icon puts it away");
        // A dialog (one a panel menu opens, say): a click in it or on its backdrop keeps the panel.
        h.click(layers.center());
        h.app.ui.dialog = Some(crate::state::Dialog::new("move", json!({"dx": "0 pt", "dy": "0 pt"})));
        h.settle();
        let dialog = h.ctx.memory(|m| m.area_rect(egui::Id::new(("dialog", "move")))).expect("the Move dialog");
        h.click(dialog.left_top() + vec2(20.0, 30.0));
        assert_eq!(h.app.ui.open_panel.as_deref(), Some("layers"), "a click in the dialog keeps it");
        h.click(egui::pos2(dialog.left() - 40.0, dialog.center().y));
        assert_eq!(h.app.ui.open_panel.as_deref(), Some("layers"), "a click on the backdrop keeps it");
        h.app.ui.dialog = None;
        // The dock expanded: a click in its panel group puts a popped-out icon panel away too.
        h.app.run("window.collapseDock", json!({"collapsed": false})).unwrap();
        h.settle();
        let icon = h.icons()[0];
        h.click(icon.center());
        let open = h.app.ui.open_panel.clone();
        assert!(open.is_some(), "an icon panel pops out beside the expanded dock");
        h.click(egui::pos2(SCREEN.x - 100.0, 300.0));
        assert_eq!(h.app.ui.open_panel, None, "a click in the docked group puts {open:?} away");
    }

    #[test]
    fn window_panel_move_reorders_legacy_dock_tabs_and_persists_order() {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.run("file.new", json!({"width": 300, "height": 300})).unwrap();
        assert!(app.ui.docking.is_none());
        app.run("window.panel.move", json!({ "panel": "libraries", "anchor": "properties", "before": "properties" })).unwrap();
        let order: Vec<_> = floating::docked_tabs(&app.ui).map(|t| t.info().0).collect();
        assert_eq!(order, ["libraries", "properties", "layers"]);
        let saved: crate::state::UiState = serde_json::from_slice(&serde_json::to_vec(&app.ui).unwrap()).unwrap();
        assert_eq!(saved.dock_tab_order, app.ui.dock_tab_order);
    }

    #[test]
    fn partial_dock_tab_order_appends_tabs_missing_from_preferences() {
        let ui =
            crate::state::UiState { dock_tab: DockTab::Layers, dock_tab_order: vec![DockTab::Layers, DockTab::Properties], ..Default::default() };
        let order = floating::resolved_dock_tab_order(&ui);
        assert_eq!(order.len(), DockTab::ALL.len());
        assert_eq!(order[..2], [DockTab::Layers, DockTab::Properties]);
        assert_eq!(order[2], DockTab::Libraries);
    }

    #[test]
    fn the_collapse_command_sets_or_toggles_the_dock() {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        assert_eq!(app.run("window.collapseDock", json!({})).unwrap(), json!(true));
        assert!(app.ui.dock_collapsed && crate::menus::checked(&app, "window.collapseDock", &json!({})) == Some(true));
        assert_eq!(app.run("window.collapseDock", json!({"collapsed": true})).unwrap(), json!(true));
        assert!(app.run("window.collapseDock", json!({"collapsed": "yes"})).is_err());
        // Window › Layers pops Layers out of its icon while collapsed.
        app.run("window.panel", json!({"panel": "layers"})).unwrap();
        assert_eq!(app.ui.open_panel.as_deref(), Some("layers"));
        assert_eq!(crate::menus::checked(&app, "window.panel", &json!({"panel": "layers"})), Some(true));
        assert_eq!(app.run("window.collapseDock", json!({})).unwrap(), json!(false));
        assert_eq!((app.ui.open_panel.as_deref(), app.ui.dock_tab), (None, DockTab::Layers));
        app.run("window.panel", json!({"panel": "properties"})).unwrap();
        assert_eq!((app.ui.open_panel.as_deref(), app.ui.dock_tab), (None, DockTab::Properties));
        // Saved with the preferences and with a user workspace; the built-in ones expand it.
        app.run("window.collapseDock", json!({"collapsed": true})).unwrap();
        let saved: crate::UiState = serde_json::from_slice(&serde_json::to_vec(&app.ui).unwrap()).unwrap();
        assert!(saved.sanitized().dock_collapsed);
        app.run("window.workspace.new", json!({"name": "Icons"})).unwrap();
        app.run("window.workspace", json!({"name": "Essentials"})).unwrap();
        assert!(!app.ui.dock_collapsed, "Essentials shows the panels");
        app.run("window.workspace", json!({"name": "Icons"})).unwrap();
        assert!(app.ui.dock_collapsed, "the user workspace keeps them collapsed");
    }
}
