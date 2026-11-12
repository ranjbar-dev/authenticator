//! Shared semantic egui controls, with opaque vector icons and quiet focus borders.
use super::{state::Route, theme};
use eframe::egui::{self, Color32, Id, Response, Stroke, Ui};

pub fn heading(ui: &mut Ui, text: &str, level: u32) -> Response {
    let role = if level == 1 {
        theme::TITLE
    } else {
        theme::SECTION
    };
    let response = ui.add(egui::Label::new(role.text(text)).wrap());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Heading, true, text));
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_level(level as usize));
    response
}
pub fn body(ui: &mut Ui, text: &str) -> Response {
    ui.add(egui::Label::new(theme::BODY.text(text)).wrap())
}
pub fn secondary(ui: &mut Ui, text: &str) -> Response {
    ui.add(egui::Label::new(theme::CAPTION.text(text).color(theme::MUTED)).wrap())
}
pub fn toolbar<R>(ui: &mut Ui, controls: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
    ui.horizontal_wrapped(controls)
}

pub fn card<R>(ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
    egui::Frame::new()
        .fill(theme::SURFACE)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(theme::RADIUS)
        .inner_margin(theme::PADDING)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            content(ui)
        })
}

pub fn focus(ui: &Ui, response: &Response) {
    // Keyboard navigation must bring an off-screen control into the shared
    // scroll holder, including compact lock forms and enlarged app text.
    if response.gained_focus() {
        response.scroll_to_me(Some(egui::Align::Center));
    }
    if response.has_focus() {
        // Outside the filled button, against its holder; no bright halo.
        ui.painter().rect_stroke(
            response.rect.expand(theme::FOCUS_STROKE),
            theme::CONTROL_RADIUS,
            Stroke::new(theme::FOCUS_STROKE, theme::FOCUS),
            egui::StrokeKind::Outside,
        );
    }
}
#[derive(Clone, Copy)]
pub enum ButtonStyle {
    Primary,
    Secondary,
}
pub fn button(ui: &mut Ui, text: &str, style: ButtonStyle) -> Response {
    button_with_height(ui, text, style, theme::CONTROL_HEIGHT)
}
pub fn button_with_height(ui: &mut Ui, text: &str, style: ButtonStyle, height: f32) -> Response {
    let primary = matches!(style, ButtonStyle::Primary);
    let response = ui.add_sized(
        [ui.available_width(), height],
        egui::Button::new(theme::LABEL.text(text).color(if primary {
            theme::ON_BLUE
        } else {
            theme::TEXT
        }))
        .fill(if primary { theme::BLUE } else { theme::SURFACE })
        .wrap(),
    );
    focus(ui, &response);
    response
}

#[derive(Clone, Copy)]
pub enum Icon {
    Lock,
    Help,
    Plus,
    Key,
    Recovery,
    Settings,
}
pub fn paint_icon(ui: &Ui, rect: egui::Rect, icon: Icon, color: Color32) {
    let center = rect.center();
    let p = |x: f32, y: f32| center + egui::vec2(x, y) * (theme::ICON_SIZE / 20.0);
    let stroke = Stroke::new(theme::ICON_STROKE, color);
    let line = |a, b| {
        ui.painter().line_segment([a, b], stroke);
    };
    match icon {
        Icon::Plus => {
            line(p(-7.0, 0.0), p(7.0, 0.0));
            line(p(0.0, -7.0), p(0.0, 7.0));
        }
        Icon::Lock => {
            ui.painter().rect_stroke(
                egui::Rect::from_min_max(p(-7.0, -1.0), p(7.0, 9.0)),
                2.0,
                stroke,
                egui::StrokeKind::Inside,
            );
            ui.painter().add(egui::Shape::line(
                vec![
                    p(-4.0, -1.0),
                    p(-4.0, -6.0),
                    p(0.0, -8.0),
                    p(4.0, -6.0),
                    p(4.0, -1.0),
                ],
                stroke,
            ));
            line(p(0.0, 3.0), p(0.0, 6.0));
        }
        Icon::Help => {
            ui.painter().circle_stroke(center, 9.0, stroke);
            ui.painter().add(egui::Shape::line(
                vec![
                    p(-3.0, -4.0),
                    p(0.0, -6.0),
                    p(3.0, -4.0),
                    p(3.0, -1.0),
                    p(0.0, 1.0),
                    p(0.0, 3.0),
                ],
                stroke,
            ));
            ui.painter().circle_filled(p(0.0, 6.0), 1.0, color);
        }
        Icon::Key => {
            ui.painter().circle_stroke(p(-4.0, -3.0), 5.0, stroke);
            line(p(0.0, 1.0), p(8.0, 9.0));
            line(p(4.0, 5.0), p(7.0, 2.0));
            line(p(7.0, 8.0), p(10.0, 5.0));
        }
        Icon::Recovery => {
            ui.painter().rect_stroke(
                egui::Rect::from_min_max(p(-7.0, -9.0), p(7.0, 9.0)),
                2.0,
                stroke,
                egui::StrokeKind::Inside,
            );
            for y in [-4.0, 0.0, 4.0] {
                line(p(-3.0, y), p(3.0, y));
            }
        }
        Icon::Settings => {
            ui.painter().circle_stroke(center, 6.0, stroke);
            ui.painter().circle_stroke(center, 2.0, stroke);
            for (x, y) in [(0.0, 1.0), (1.0, 0.0), (-1.0, 0.0), (0.0, -1.0)] {
                line(p(x * 6.0, y * 6.0), p(x * 10.0, y * 10.0));
            }
        }
    }
}
pub fn icon_button(ui: &mut Ui, icon: Icon, name: &str) -> Response {
    let response = ui.add_sized(
        [theme::CONTROL_HEIGHT; 2],
        egui::Button::new("").fill(theme::BLUE),
    );
    paint_icon(ui, response.rect, icon, theme::ON_BLUE);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), name));
    focus(ui, &response);
    response.on_hover_text(name)
}
pub fn icon_tile(ui: &mut Ui, icon: Icon) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(theme::CONTROL_HEIGHT, theme::CONTROL_HEIGHT),
        egui::Sense::hover(),
    );
    ui.painter()
        .rect_filled(rect, theme::CONTROL_RADIUS, theme::BLUE);
    paint_icon(ui, rect, icon, theme::ON_BLUE);
}

pub fn tabs(ui: &mut Ui, selected: Route) -> Option<Route> {
    let mut next = None;
    // Stack at very small logical widths (e.g. 200% zoom) rather than truncate.
    let compact = ui.available_width() < theme::COMPACT_TABS_WIDTH;
    let width = if compact {
        ui.available_width()
    } else {
        ((ui.available_width() - theme::SMALL_GAP * 2.0) / 3.0).floor()
    };
    let height = if !compact && ui.available_width() < theme::TABS_SINGLE_LINE_WIDTH {
        theme::WRAPPED_TAB_HEIGHT
    } else {
        theme::CONTROL_HEIGHT
    };
    let render = |ui: &mut Ui| {
        for route in Route::ALL {
            let active = route == selected;
            let response = ui.add_sized(
                [width, height],
                egui::Button::new(theme::LABEL.text(route.title()).color(if active {
                    theme::ON_BLUE
                } else {
                    theme::TEXT
                }))
                .fill(if active { theme::BLUE } else { theme::SURFACE })
                .selected(active)
                .wrap(),
            );
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::Role::Tab, true, active, route.title())
            });
            focus(ui, &response);
            if response.clicked() {
                next = Some(route);
            }
        }
    };
    if compact {
        ui.vertical(render);
    } else {
        ui.horizontal(render);
    }
    next
}

pub fn labeled_field(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    label: &str,
    value: &mut String,
    helper: &str,
    error: Option<&str>,
) -> Response {
    labeled_field_impl(ui, id, label, value, helper, error, FieldMode::Plain)
}
pub fn labeled_search_field(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    value: &mut String,
    error: Option<&str>,
) -> Response {
    labeled_field_impl(
        ui,
        id,
        "Search accounts",
        value,
        "",
        error,
        FieldMode::Search,
    )
}

/// Explicit secret adapter: native password semantics AND a masked display
/// layouter. A custom TextEdit layouter bypasses egui's default masking.
pub fn labeled_secret_field(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    label: &str,
    value: &mut String,
    helper: &str,
    error: Option<&str>,
) -> Response {
    labeled_field_impl(ui, id, label, value, helper, error, FieldMode::Secret)
}
pub fn labeled_revealed_secret_field(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    label: &str,
    value: &mut String,
    helper: &str,
    error: Option<&str>,
) -> Response {
    labeled_field_impl(ui, id, label, value, helper, error, FieldMode::Revealed)
}

/// Release all native text-editor histories at the session/form boundary.
/// Egui does not offer zeroizing undo allocations; dropping history is best
/// effort lifetime minimization, not a guarantee about allocator/OS copies.
pub fn clear_sensitive_widget_memory(ctx: &egui::Context) {
    ctx.memory_mut(|m| m.data.clear());
}

#[derive(Clone, Copy)]
enum FieldMode {
    Plain,
    Search,
    Secret,
    Revealed,
}

/// Full-role, labeled native selector, including current state and focus.
pub fn labeled_choice(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    label: &str,
    value: &mut String,
    choices: &[&str],
) -> bool {
    ui.push_id(id, |ui| {
        let label = ui.label(theme::LABEL.text(label));
        let mut changed = false;
        let response = egui::ComboBox::from_id_salt("choice")
            .truncate()
            .selected_text(theme::BODY.text(value.as_str()).color(theme::TEXT))
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for choice in choices {
                    let selected = value == choice;
                    let response = ui.selectable_label(
                        selected,
                        theme::BODY.text(*choice).color(if selected {
                            theme::ON_BLUE
                        } else {
                            theme::TEXT
                        }),
                    );
                    focus(ui, &response);
                    if response.clicked() {
                        *value = (*choice).into();
                        changed = true;
                        ui.close();
                    }
                }
            })
            .response
            .labelled_by(label.id);
        if changed {
            response.request_focus();
        }
        focus(ui, &response);
        changed
    })
    .inner
}

/// Stable-value counterpart for folders whose visible names may be identical.
pub fn labeled_selection<T: Copy + Eq + std::fmt::Debug>(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    label: &str,
    value: &mut T,
    choices: &[(T, &str)],
) -> bool {
    ui.push_id(id, |ui| {
        let label = ui.label(theme::LABEL.text(label));
        let mut changed = false;
        let selected_text = choices
            .iter()
            .find(|(id, _)| id == value)
            .map(|(_, text)| *text)
            .unwrap_or("Selected folder");
        let response = egui::ComboBox::from_id_salt("choice")
            .truncate()
            .selected_text(theme::BODY.text(selected_text).color(theme::TEXT))
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (index, (id, text)) in choices.iter().enumerate() {
                    ui.push_id(index, |ui| {
                        let selected = id == value;
                        let response = ui.selectable_label(
                            selected,
                            theme::BODY.text(*text).color(if selected {
                                theme::ON_BLUE
                            } else {
                                theme::TEXT
                            }),
                        );
                        focus(ui, &response);
                        if response.clicked() {
                            *value = *id;
                            changed = true;
                            ui.close();
                        }
                    });
                }
            })
            .response
            .labelled_by(label.id);
        if changed {
            response.request_focus();
        }
        focus(ui, &response);
        changed
    })
    .inner
}

/// Recovery editor keeps newlines while masking each non-newline scalar. Native
/// password semantics block clipboard text output in both modes. Concealed
/// layout/AX contain only masks; deliberate Reveal exposes visual/AX text.
/// Undo history is released every pass.
pub fn labeled_sensitive_multiline(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    label: &str,
    value: &mut String,
    reveal: bool,
) -> Response {
    ui.push_id(id, |ui| {
        let label = ui.label(theme::LABEL.text(label));
        let id = ui.scope_id().with("input");
        let mut layouter = |ui: &Ui, buffer: &dyn egui::TextBuffer, width: f32| {
            let display = if reveal {
                buffer.as_str().to_owned()
            } else {
                buffer
                    .as_str()
                    .chars()
                    .map(|c| {
                        if c == '\n' {
                            c
                        } else {
                            egui::epaint::text::PASSWORD_REPLACEMENT_CHAR
                        }
                    })
                    .collect()
            };
            let mut job = theme::BODY.singleline_job(display, theme::TEXT);
            job.break_on_newline = true;
            job.wrap.max_width = width;
            ui.fonts_mut(|f| f.layout_job(job))
        };
        let scroll = egui::ScrollArea::vertical()
            .id_salt("sensitive-text-viewport")
            .max_height(theme::MULTILINE_HEIGHT)
            .min_scrolled_height(theme::MULTILINE_HEIGHT)
            .auto_shrink([false, true])
            .scroll_source(egui::scroll_area::ScrollSource {
                drag: egui::scroll_area::DragScroll::Never,
                ..Default::default()
            })
            .show(ui, |ui| {
                egui::TextEdit::multiline(value)
                    .id(id)
                    .font(theme::BODY.font())
                    .password(true)
                    // Retain one scalar ABOVE the byte limit: native input
                    // clamping must not turn an oversized paste into accepted
                    // truncation. Parse still rejects >1MiB UTF-8 bytes.
                    .char_limit(crate::domain::recovery_input::MAX_TEXT_BYTES + 1)
                    .layouter(&mut layouter)
                    .desired_rows(5)
                    .desired_width(ui.available_width())
                    .show(ui)
            });
        let mut output = scroll.inner;
        output.state.clear_undoer();
        output.state.clone().store(ui.ctx(), id);
        let mut response = output.response.response.labelled_by(label.id);
        // Focus/outer form scrolling refer to the visible editor, rather than
        // the potentially thousands-of-lines native input's full content rect.
        response.rect = response.rect.intersect(scroll.inner_rect);
        response.interact_rect = response.interact_rect.intersect(scroll.inner_rect);
        let global_rect = ui
            .ctx()
            .layer_transform_to_global(ui.layer_id())
            .map_or(response.rect, |transform| transform * response.rect);
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_bounds(egui::accesskit::Rect {
                x0: global_rect.min.x.into(),
                y0: global_rect.min.y.into(),
                x1: global_rect.max.x.into(),
                y1: global_rect.max.y.into(),
            })
        });
        ui.painter().rect_stroke(
            response.rect,
            theme::CONTROL_RADIUS,
            Stroke::new(1., theme::BORDER),
            egui::StrokeKind::Inside,
        );
        focus(ui, &response);
        response
    })
    .inner
}

fn field_editor(
    ui: &mut Ui,
    id: Id,
    value: &mut String,
    mode: FieldMode,
    typography: theme::Typography,
) -> egui::text_edit::TextEditOutput {
    let secret = !matches!(mode, FieldMode::Plain | FieldMode::Search);
    let masked = matches!(mode, FieldMode::Secret);
    let mut layouter = |ui: &Ui, buffer: &dyn egui::TextBuffer, _wrap_width: f32| {
        // Concealed mode never hands raw secret text to retained font layout.
        // Deliberate reveal does, while keeping native clipboard protection and a password parent role;
        // framework/allocator/OS layout copies cannot be guaranteed erased.
        // Match egui's one replacement character per Unicode scalar so native
        // cursor/selection character indexes still map to the displayed text.
        let display = if masked {
            std::iter::repeat_n(
                egui::epaint::text::PASSWORD_REPLACEMENT_CHAR,
                buffer.as_str().chars().count(),
            )
            .collect()
        } else {
            buffer.as_str().to_owned()
        };
        ui.fonts_mut(|fonts| fonts.layout_job(typography.singleline_job(display, theme::TEXT)))
    };
    let layout = egui::Layout::centered_and_justified(ui.layout().main_dir());
    let mut output = ui
        .allocate_ui_with_layout(
            egui::vec2(ui.available_width(), theme::CONTROL_HEIGHT),
            layout,
            |ui| {
                egui::TextEdit::singleline(value)
                    .char_limit(if matches!(mode, FieldMode::Search) {
                        crate::domain::records::CodesPreferences::MAX_QUERY_BYTES + 1
                    } else {
                        usize::MAX
                    })
                    .font(typography.font())
                    .password(secret)
                    .layouter(&mut layouter)
                    .id(id)
                    .show(ui)
            },
        )
        .inner;
    if secret {
        // Native password rendering does not disable egui's plaintext undo
        // history. Never retain that history between secret-editor passes.
        output.state.clear_undoer();
        output.state.clone().store(ui.ctx(), id);
    }
    output
}

fn labeled_field_impl(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    label: &str,
    value: &mut String,
    helper: &str,
    error: Option<&str>,
    mode: FieldMode,
) -> Response {
    ui.push_id(id, |ui| {
        let label = ui.label(theme::LABEL.text(label));
        let response = field_editor(ui, ui.scope_id().with("input"), value, mode, theme::BODY)
            .response
            .response
            .labelled_by(label.id);
        let detail = (!helper.is_empty() || error.is_some()).then(|| {
            ui.add(
                egui::Label::new(theme::CAPTION.text(error.unwrap_or(helper)).color(
                    if error.is_some() {
                        theme::ERROR
                    } else {
                        theme::MUTED
                    },
                ))
                .wrap(),
            )
        });
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_description(error.unwrap_or(helper));
            if let Some(detail) = &detail {
                node.push_described_by(detail.id.accesskit_id());
            }
            if error.is_some() {
                node.set_invalid(egui::accesskit::Invalid::True);
            }
        });
        focus(ui, &response);
        response
    })
    .inner
}

#[cfg(test)]
mod field_tests {
    use super::*;
    #[test]
    fn narrow_tabs_keep_full_labels_equal_wrapped_height_and_one_keyboard_row() {
        let ctx = egui::Context::default();
        theme::apply(&ctx);
        ctx.enable_accesskit();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(340., 780.),
                )),
                ..Default::default()
            },
            |root| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().inner_margin(theme::PADDING))
                    .show(root, |ui| {
                        tabs(ui, Route::Codes);
                    });
            },
        );
        output.textures_delta.clear();
        let Some(tree) = output.platform_output.accesskit_update.as_ref() else {
            panic!("accessibility test output")
        };
        let tabs = tree
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == egui::Role::Tab)
            .collect::<Vec<_>>();
        assert_eq!(tabs.len(), 3);
        let mut positions = Vec::new();
        for (_, node) in tabs {
            assert!(
                Route::ALL
                    .iter()
                    .any(|route| Some(route.title()) == node.label())
            );
            let Some(bounds) = node.bounds() else {
                panic!("tab bounds")
            };
            assert!((bounds.y1 - bounds.y0 - f64::from(theme::WRAPPED_TAB_HEIGHT)).abs() < 0.1);
            positions.push(bounds);
        }
        positions.sort_by(|a, b| a.x0.total_cmp(&b.x0));
        assert!(
            positions
                .iter()
                .all(|bounds| (bounds.y0 - positions[0].y0).abs() < 0.1)
        );
        assert!(
            positions
                .windows(2)
                .all(|bounds| bounds[0].x1 <= bounds[1].x0)
        );
        assert!(positions[2].x1 - positions[0].x0 <= 308.1, "{positions:?}");
        assert!(
            output
                .shapes
                .iter()
                .filter_map(|shape| match &shape.shape {
                    egui::Shape::Text(text)
                        if Route::ALL
                            .iter()
                            .any(|route| route.title() == text.galley.job.text) =>
                        Some(&text.galley),
                    _ => None,
                })
                .all(|galley| galley
                    .job
                    .sections
                    .iter()
                    .all(|section| section.format.font_id == theme::LABEL.font()
                        && section.format.line_height == Some(theme::LABEL.line_height)))
        );
    }
    #[test]
    fn many_recovery_lines_keep_the_editor_viewport_and_save_compact() {
        let ctx = egui::Context::default();
        theme::apply(&ctx);
        ctx.enable_accesskit();
        let mut value = (0..40)
            .map(|i| format!("public-fixture-{i}\n"))
            .collect::<String>();
        let mut frame = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(340., 600.),
                )),
                ..Default::default()
            },
            |root| {
                egui::CentralPanel::default().show(root, |ui| {
                    let field = labeled_sensitive_multiline(
                        ui,
                        "many-lines",
                        "Backup codes",
                        &mut value,
                        false,
                    );
                    assert!(field.rect.height() <= theme::MULTILINE_HEIGHT);
                    let save = button(ui, "Save", ButtonStyle::Primary);
                    assert!(
                        save.rect.top()
                            < field.rect.top() + theme::MULTILINE_HEIGHT + theme::GAP * 2.
                    );
                    assert!(save.rect.bottom() < 300.);
                });
            },
        );
        frame.textures_delta.clear();
        assert_eq!(value.lines().count(), 40);
        assert!(
            frame
                .platform_output
                .accesskit_update
                .as_ref()
                .is_some_and(|tree| tree.nodes.iter().any(|(_, node)| node.role()
                    == egui::Role::PasswordInput
                    && node.bounds().is_some_and(
                        |rect| rect.y1 - rect.y0 <= f64::from(theme::MULTILINE_HEIGHT)
                    )))
        );
    }
    #[test]
    fn password_clipboard_output_is_blocked_and_reveal_ax_exposure_is_explicit() {
        for mode in [FieldMode::Secret, FieldMode::Revealed] {
            let ctx = egui::Context::default();
            theme::apply(&ctx);
            ctx.enable_accesskit();
            let id = Id::unique("clipboard-fixture");
            let mut value = "sentinel-visible-".repeat(40);
            let mut frame = ctx.run_ui(egui::RawInput::default(), |root| {
                egui::CentralPanel::default().show(root, |ui| {
                    let mut output = field_editor(ui, id, &mut value, mode, theme::BODY);
                    output.response.request_focus();
                    output
                        .state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::two(
                            egui::text::CCursor::new(0),
                            egui::text::CCursor::new(value.chars().count()),
                        )));
                    output.state.store(ui.ctx(), id);
                });
            });
            frame.textures_delta.clear();
            let raw_runs = frame
                .platform_output
                .accesskit_update
                .as_ref()
                .is_some_and(|tree| {
                    tree.nodes
                        .iter()
                        .any(|(_, node)| node.value().is_some_and(|v| v.contains("sentinel")))
                });
            assert_eq!(
                raw_runs,
                matches!(mode, FieldMode::Revealed),
                "Deliberate reveal exposes text runs; concealed mode must not"
            );
            for event in [egui::Event::Copy, egui::Event::Cut] {
                let input = egui::RawInput {
                    events: vec![event],
                    ..Default::default()
                };
                let mut frame = ctx.run_ui(input, |root| {
                    egui::CentralPanel::default().show(root, |ui| {
                        field_editor(ui, id, &mut value, mode, theme::BODY);
                    });
                });
                frame.textures_delta.clear();
                assert!(
                    !frame
                        .platform_output
                        .commands
                        .iter()
                        .any(|c| matches!(c, egui::OutputCommand::CopyText(_)))
                );
            }
            // Reconceal emits masked values for every remaining text run.
            value = "sentinel-visible-".repeat(40);
            let mut hidden = ctx.run_ui(egui::RawInput::default(), |root| {
                egui::CentralPanel::default().show(root, |ui| {
                    field_editor(ui, id, &mut value, FieldMode::Secret, theme::BODY);
                });
            });
            hidden.textures_delta.clear();
            assert!(
                hidden
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .is_some_and(|tree| tree
                        .nodes
                        .iter()
                        .all(|(_, node)| node.value().is_none_or(|v| !v.contains("sentinel"))))
            );
        }
    }

    #[test]
    fn multiline_conceals_full_ax_and_uses_complete_wrapped_role_metrics() {
        let ctx = egui::Context::default();
        theme::apply(&ctx);
        ctx.enable_accesskit();
        let mut value = "sentinel-recovery-".repeat(40) + "\nsecond-recovery";
        for reveal in [false, true, false] {
            let mut output = ctx.run_ui(egui::RawInput::default(), |root| {
                egui::CentralPanel::default().show(root, |ui| {
                    ui.set_width(240.);
                    labeled_sensitive_multiline(
                        ui,
                        "recovery-probe",
                        "Backup codes",
                        &mut value,
                        reveal,
                    );
                });
            });
            output.textures_delta.clear();
            let galleys = output
                .shapes
                .iter()
                .filter_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.job.text.chars().count() > 600 => {
                        Some(&text.galley)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert!(!galleys.is_empty());
            for galley in galleys {
                assert!(galley.rows.len() > 1);
                assert!(
                    galley
                        .job
                        .sections
                        .iter()
                        .all(|section| section.format.font_id == theme::BODY.font()
                            && section.format.line_height == Some(theme::BODY.line_height)
                            && section.format.extra_letter_spacing == theme::BODY.letter_spacing)
                );
                assert!(
                    galley
                        .rows
                        .iter()
                        .all(|row| (row.size.y - theme::BODY.line_height).abs() < 0.01)
                );
            }
            let raw = output
                .platform_output
                .accesskit_update
                .as_ref()
                .is_some_and(|tree| {
                    tree.nodes
                        .iter()
                        .any(|(_, n)| n.value().is_some_and(|v| v.contains("sentinel")))
                });
            assert_eq!(raw, reveal);
        }
    }

    #[test]
    fn native_editor_uses_complete_typography_and_measured_row_metrics() {
        let ctx = egui::Context::default();
        theme::apply(&ctx);
        let mut frame = ctx.run_ui(egui::RawInput::default(), |root| {
            egui::CentralPanel::default().show(root, |ui| {
                let mut value = "MMMMMMMM ".to_owned();
                let body = field_editor(
                    ui,
                    Id::unique("body"),
                    &mut value,
                    FieldMode::Plain,
                    theme::BODY,
                );
                assert_eq!(body.galley.job.text, value);
                assert_eq!(body.galley.rows.len(), 1);
                assert!((body.galley.rows[0].size.y - theme::BODY.line_height).abs() < 0.01);
                let format = &body.galley.job.sections[0].format;
                assert_eq!(format.font_id, theme::BODY.font());
                assert_eq!(format.extra_letter_spacing, theme::BODY.letter_spacing);
                // Nonzero tracking and another line height must affect actual
                // layout, not just be metadata on an ignored role bundle.
                let varied = theme::Typography {
                    line_height: 28.0,
                    letter_spacing: 2.0,
                    ..theme::BODY
                };
                let tracked = field_editor(
                    ui,
                    Id::unique("tracked"),
                    &mut value,
                    FieldMode::Plain,
                    varied,
                );
                assert!((tracked.galley.rows[0].size.y - 28.0).abs() < 0.01);
                assert!(tracked.galley.rect.width() > body.galley.rect.width() + 10.0);
                assert_eq!(tracked.galley.job.text, "MMMMMMMM ");
            });
        });
        frame.textures_delta.clear();
    }

    #[test]
    fn secret_editor_galley_contains_only_masked_unicode_characters() {
        let ctx = egui::Context::default();
        theme::apply(&ctx);
        ctx.enable_accesskit();
        let mut frame = ctx.run_ui(egui::RawInput::default(), |root| {
            egui::CentralPanel::default().show(root, |ui| {
                let mut value = "disposable-é🦀".to_owned();
                let output = field_editor(
                    ui,
                    Id::unique("secret"),
                    &mut value,
                    FieldMode::Secret,
                    theme::BODY,
                );
                let count = value.chars().count();
                assert_eq!(output.galley.job.text.chars().count(), count);
                assert!(
                    output
                        .galley
                        .job
                        .text
                        .chars()
                        .all(|c| c == egui::epaint::text::PASSWORD_REPLACEMENT_CHAR)
                );
                assert!(!output.galley.job.text.contains("disposable"));
                assert_eq!(output.galley.rows[0].glyphs.len(), count);
                assert!((output.galley.rows[0].size.y - theme::BODY.line_height).abs() < 0.01);
                assert_eq!(value, "disposable-é🦀");
            });
        });
        frame.textures_delta.clear();
        let update = frame.platform_output.accesskit_update.as_ref();
        assert!(update.is_some_and(|tree| {
            tree.nodes
                .iter()
                .any(|(_, node)| node.role() == egui::Role::PasswordInput)
        }));
        if let Some(tree) = update {
            assert!(
                tree.nodes
                    .iter()
                    .all(|(_, node)| node.value() != Some("disposable-é🦀"))
            );
        }
    }
}

#[derive(Clone, Copy)]
pub enum StatusKind {
    Info,
    Success,
    Error,
}
pub fn status(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    text: &str,
    kind: StatusKind,
) {
    ui.push_id(id, |ui| {
        let (color, fill) = match kind {
            StatusKind::Info => (theme::MUTED, theme::SURFACE),
            StatusKind::Success => (theme::SUCCESS, theme::SURFACE),
            StatusKind::Error => (theme::ERROR, theme::ERROR_SURFACE),
        };
        egui::Frame::new()
            .fill(fill)
            .corner_radius(theme::CONTROL_RADIUS)
            .inner_margin(theme::PADDING)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let response = ui.add(egui::Label::new(theme::BODY.text(text).color(color)).wrap());
                response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Status, true, text));
                ui.ctx().accesskit_node_builder(response.id, |node| {
                    node.set_live(egui::accesskit::Live::Polite)
                });
            });
    });
}
pub fn empty_state(ui: &mut Ui, icon: Icon, title: &str, description: &str) {
    card(ui, |ui| {
        icon_tile(ui, icon);
        heading(ui, title, 2);
        body(ui, description);
    });
}
pub fn progress(ui: &mut Ui, label: &str, fraction: Option<f32>) {
    body(ui, label);
    if let Some(fraction) = fraction {
        let response = ui.add(
            egui::ProgressBar::new(fraction.clamp(0.0, 1.0))
                .desired_height(theme::PROGRESS_HEIGHT)
                .fill(theme::BLUE),
        );
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::ProgressIndicator, true, label));
    } else {
        ui.horizontal(|ui| {
            let response = ui.spinner();
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::ProgressIndicator, true, label)
            });
            secondary(ui, "Please wait…");
        });
    }
}

/// In-root modal so it uses ROOT's initialized AccessKit adapter.
pub struct Dialog {
    pub title: &'static str,
    pub opener: Id,
    first_frame: bool,
}
/// Call at the beginning of the root UI pass. Egui retains the previous modal
/// layer for one pass; restoring earlier would immediately surrender focus.
pub fn restore_dialog_focus(ctx: &egui::Context) {
    let key = Id::unique("dialog-return-focus");
    if let Some(opener) = ctx.data(|data| data.get_temp::<Id>(key)) {
        if ctx.memory(|m| m.top_modal_layer().is_none()) {
            ctx.memory_mut(|m| m.request_focus(opener));
            ctx.data_mut(|data| data.remove::<Id>(key));
        } else {
            ctx.request_repaint();
        }
    }
}
impl Dialog {
    pub fn new(title: &'static str, opener: Id) -> Self {
        Self {
            title,
            opener,
            first_frame: true,
        }
    }
    /// `content` returns true to close. Escape/backdrop also close and restore focus.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        content: impl FnOnce(&mut Ui, bool) -> bool,
    ) -> bool {
        let first = self.first_frame;
        let modal = egui::Modal::new(Id::unique("shared-dialog"))
            .area(
                egui::Modal::default_area(Id::unique("shared-dialog")).accessible_name(self.title),
            )
            .backdrop_color(theme::SCRIM)
            .show(ctx, |ui| {
                ui.set_width(
                    theme::DIALOG_WIDTH.min(
                        (ctx.content_rect().width() - f32::from(theme::PADDING) * 2.0)
                            .max(theme::CONTROL_HEIGHT),
                    ),
                );
                heading(ui, self.title, 2);
                egui::ScrollArea::vertical()
                    .max_height((ctx.content_rect().height() - 120.0).max(theme::CONTROL_HEIGHT))
                    .show(ui, |ui| content(ui, first))
                    .inner
            });
        self.first_frame = false;
        let close = modal.inner || modal.should_close();
        if close {
            ctx.data_mut(|data| data.insert_temp(Id::unique("dialog-return-focus"), self.opener));
            ctx.request_repaint();
        }
        close
    }
}

#[cfg(test)]
mod secret_history_tests {
    use super::*;
    #[test]
    fn secret_editor_clears_old_undo_and_boundary_removes_widget_state() {
        let ctx = egui::Context::default();
        theme::apply(&ctx);
        let id = Id::unique("secret-history-probe");
        let current = (
            egui::text::CCursorRange::default(),
            "disposable previous text".to_owned(),
        );
        let mut old = egui::text_edit::TextEditState::default();
        let mut undo = old.undoer();
        undo.add_undo(&current);
        old.set_undoer(undo);
        old.store(&ctx, id);
        let mut output = ctx.run_ui(egui::RawInput::default(), |root| {
            egui::CentralPanel::default().show(root, |ui| {
                let mut value = "disposable current text".to_owned();
                let output = field_editor(ui, id, &mut value, FieldMode::Secret, theme::BODY);
                assert!(!output.state.undoer().has_undo(&current));
                assert!(!output.state.undoer().is_in_flux());
            });
        });
        output.textures_delta.clear();
        assert!(egui::text_edit::TextEditState::load(&ctx, id).is_some());
        clear_sensitive_widget_memory(&ctx);
        assert!(egui::text_edit::TextEditState::load(&ctx, id).is_none());
    }
}
