use crate::inspector::editors::spritesheet::SpriteSheetFramesPropertyEditorMessage;
use fyrox::{
    animation::spritesheet::SpriteSheetFramesContainer,
    core::{algebra::Vector2, color::Color, pool::Handle},
    gui::{
        border::BorderBuilder,
        brush::Brush,
        button::{ButtonBuilder, ButtonMessage},
        check_box::{CheckBoxBuilder, CheckBoxMessage},
        draw::DrawingContext,
        grid::{Column, GridBuilder, Row},
        image::ImageBuilder,
        message::{MessageDirection, OsEvent, UiMessage},
        numeric::{NumericUpDownBuilder, NumericUpDownMessage},
        stack_panel::StackPanelBuilder,
        text::TextBuilder,
        widget::{Widget, WidgetBuilder, WidgetMessage},
        window::{Window, WindowBuilder, WindowMessage, WindowTitle},
        BuildContext, Control, HorizontalAlignment, NodeHandleMapping, Orientation, Thickness,
        UiNode, UserInterface, VerticalAlignment,
    },
    utils::into_gui_texture,
};
use std::{
    any::{Any, TypeId},
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::mpsc::Sender,
};

#[derive(Clone)]
pub struct SpriteSheetFramesEditorWindow {
    window: Window,
    container: SpriteSheetFramesContainer,
    editor: Handle<UiNode>,
    ok: Handle<UiNode>,
    cancel: Handle<UiNode>,
    width: Handle<UiNode>,
    height: Handle<UiNode>,
    grid: Handle<UiNode>,
    preview_container: Handle<UiNode>,
    cells: Vec<Handle<UiNode>>,
}

impl Deref for SpriteSheetFramesEditorWindow {
    type Target = Widget;

    fn deref(&self) -> &Self::Target {
        &self.window.widget
    }
}

impl DerefMut for SpriteSheetFramesEditorWindow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.window.widget
    }
}

impl Control for SpriteSheetFramesEditorWindow {
    fn query_component(&self, type_id: TypeId) -> Option<&dyn Any> {
        self.window.query_component(type_id).or_else(|| {
            if type_id == TypeId::of::<Self>() {
                Some(self)
            } else {
                None
            }
        })
    }

    fn resolve(&mut self, node_map: &NodeHandleMapping) {
        self.window.resolve(node_map);
    }

    fn on_remove(&self, sender: &Sender<UiMessage>) {
        self.window.on_remove(sender);
    }

    fn measure_override(&self, ui: &UserInterface, available_size: Vector2<f32>) -> Vector2<f32> {
        self.window.measure_override(ui, available_size)
    }

    fn arrange_override(&self, ui: &UserInterface, final_size: Vector2<f32>) -> Vector2<f32> {
        self.window.arrange_override(ui, final_size)
    }

    fn draw(&self, drawing_context: &mut DrawingContext) {
        self.window.draw(drawing_context)
    }

    fn update(&mut self, dt: f32, sender: &Sender<UiMessage>) {
        self.window.update(dt, sender);
    }

    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        self.window.handle_routed_message(ui, message);

        if let Some(WindowMessage::Close) = message.data() {
            if message.destination() == self.handle {
                ui.send_message(WidgetMessage::remove(
                    self.handle,
                    MessageDirection::ToWidget,
                ));
            }
        } else if let Some(ButtonMessage::Click) = message.data() {
            if message.destination() == self.ok {
                ui.send_message(WindowMessage::close(
                    self.handle,
                    MessageDirection::ToWidget,
                ));

                ui.send_message(SpriteSheetFramesPropertyEditorMessage::value(
                    self.editor,
                    MessageDirection::FromWidget,
                    self.container.clone(),
                ));
            } else if message.destination() == self.cancel {
                ui.send_message(WindowMessage::close(
                    self.handle,
                    MessageDirection::ToWidget,
                ));
            }
        } else if let Some(NumericUpDownMessage::Value(value)) = message.data() {
            if message.destination() == self.width {
                let height = self.container.size().y;
                self.resize(Vector2::new(*value, height), ui);
            } else if message.destination() == self.height {
                let width = self.container.size().x;
                self.resize(Vector2::new(width, *value), ui);
            }
        } else if let Some(CheckBoxMessage::Check(Some(value))) = message.data() {
            if self.cells.contains(&message.destination())
                && message.direction == MessageDirection::FromWidget
            {
                let cell_position = ui
                    .node(message.destination())
                    .user_data_ref::<Vector2<u32>>()
                    .unwrap();

                if *value {
                    self.container.push(*cell_position);
                } else {
                    let position = self.container.iter().position(|p| p == cell_position);
                    if let Some(i) = position {
                        self.container.remove(i);
                    }
                }

                self.container.sort_by_position();
            }
        }
    }

    fn preview_message(&self, ui: &UserInterface, message: &mut UiMessage) {
        self.window.preview_message(ui, message)
    }

    fn handle_os_event(
        &mut self,
        self_handle: Handle<UiNode>,
        ui: &mut UserInterface,
        event: &OsEvent,
    ) {
        self.window.handle_os_event(self_handle, ui, event)
    }
}

fn make_grid(
    ctx: &mut BuildContext,
    container: &SpriteSheetFramesContainer,
) -> (Handle<UiNode>, Vec<Handle<UiNode>>) {
    let mut cells = Vec::new();
    for i in 0..container.size().y {
        for j in 0..container.size().x {
            let cell_position = Vector2::new(j, i);

            cells.push(
                CheckBoxBuilder::new(
                    WidgetBuilder::new()
                        .with_vertical_alignment(VerticalAlignment::Top)
                        .with_horizontal_alignment(HorizontalAlignment::Right)
                        .with_margin(Thickness::uniform(1.0))
                        .with_width(16.0)
                        .with_height(16.0)
                        .on_row(i as usize)
                        .on_column(j as usize)
                        .with_user_data(Rc::new(cell_position)),
                )
                .checked(Some(container.iter().any(|pos| *pos == cell_position)))
                .build(ctx),
            )
        }
    }

    let grid = GridBuilder::new(
        WidgetBuilder::new()
            .with_margin(Thickness::uniform(1.0))
            .with_foreground(Brush::Solid(Color::opaque(127, 127, 127)))
            .with_children(cells.clone()),
    )
    .add_columns((0..container.size().x).map(|_| Column::stretch()).collect())
    .add_rows((0..container.size().y).map(|_| Row::stretch()).collect())
    .draw_border(true)
    .build(ctx);

    (grid, cells)
}

impl SpriteSheetFramesEditorWindow {
    fn resize(&mut self, size: Vector2<u32>, ui: &mut UserInterface) {
        self.container.set_size(size);

        ui.send_message(WidgetMessage::remove(self.grid, MessageDirection::ToWidget));

        let (grid, cells) = make_grid(&mut ui.build_ctx(), &self.container);

        self.grid = grid;
        self.cells = cells;

        ui.send_message(WidgetMessage::link(
            self.grid,
            MessageDirection::ToWidget,
            self.preview_container,
        ));
    }

    pub fn build(
        ctx: &mut BuildContext,
        container: SpriteSheetFramesContainer,
        editor: Handle<UiNode>,
    ) -> Handle<UiNode> {
        let ok;
        let cancel;
        let width;
        let height;
        let preview_container;
        let (grid, cells) = make_grid(ctx, &container);
        let editor = Self {
            window: WindowBuilder::new(WidgetBuilder::new().with_width(300.0).with_height(400.0))
                .can_resize(true)
                .with_content(
                    GridBuilder::new(
                        WidgetBuilder::new()
                            .with_child(
                                GridBuilder::new(
                                    WidgetBuilder::new()
                                        .on_column(0)
                                        .on_row(0)
                                        .with_child(
                                            TextBuilder::new(
                                                WidgetBuilder::new()
                                                    .on_column(0)
                                                    .on_row(0)
                                                    .with_vertical_alignment(
                                                        VerticalAlignment::Center,
                                                    ),
                                            )
                                            .with_text("Width")
                                            .build(ctx),
                                        )
                                        .with_child({
                                            width = NumericUpDownBuilder::new(
                                                WidgetBuilder::new().on_column(1).on_row(0),
                                            )
                                            .with_min_value(0)
                                            .with_value(container.size().x)
                                            .build(ctx);
                                            width
                                        })
                                        .with_child(
                                            TextBuilder::new(
                                                WidgetBuilder::new()
                                                    .on_column(2)
                                                    .on_row(0)
                                                    .with_vertical_alignment(
                                                        VerticalAlignment::Center,
                                                    ),
                                            )
                                            .with_text("Height")
                                            .build(ctx),
                                        )
                                        .with_child({
                                            height = NumericUpDownBuilder::new(
                                                WidgetBuilder::new().on_column(3).on_row(0),
                                            )
                                            .with_min_value(0)
                                            .with_value(container.size().y)
                                            .build(ctx);
                                            height
                                        }),
                                )
                                .add_row(Row::auto())
                                .add_column(Column::stretch())
                                .add_column(Column::stretch())
                                .add_column(Column::stretch())
                                .add_column(Column::stretch())
                                .build(ctx),
                            )
                            .with_child({
                                preview_container = BorderBuilder::new(
                                    WidgetBuilder::new()
                                        .with_margin(Thickness::uniform(1.0))
                                        .on_column(0)
                                        .on_row(1)
                                        .with_child(
                                            ImageBuilder::new(
                                                WidgetBuilder::new()
                                                    .with_margin(Thickness::uniform(1.0)),
                                            )
                                            .with_opt_texture(
                                                container.texture().map(into_gui_texture),
                                            )
                                            .build(ctx),
                                        )
                                        .with_child(grid),
                                )
                                .build(ctx);
                                preview_container
                            })
                            .with_child(
                                StackPanelBuilder::new(
                                    WidgetBuilder::new()
                                        .on_column(0)
                                        .on_row(2)
                                        .with_horizontal_alignment(HorizontalAlignment::Right)
                                        .with_child({
                                            ok = ButtonBuilder::new(
                                                WidgetBuilder::new()
                                                    .with_width(100.0)
                                                    .with_margin(Thickness::uniform(1.0)),
                                            )
                                            .with_text("OK")
                                            .build(ctx);
                                            ok
                                        })
                                        .with_child({
                                            cancel = ButtonBuilder::new(
                                                WidgetBuilder::new()
                                                    .with_width(100.0)
                                                    .with_margin(Thickness::uniform(1.0)),
                                            )
                                            .with_text("Cancel")
                                            .build(ctx);
                                            cancel
                                        }),
                                )
                                .with_orientation(Orientation::Horizontal)
                                .build(ctx),
                            ),
                    )
                    .add_row(Row::strict(25.0))
                    .add_row(Row::stretch())
                    .add_row(Row::strict(25.0))
                    .add_column(Column::stretch())
                    .build(ctx),
                )
                .open(false)
                .can_minimize(false)
                .with_title(WindowTitle::text("Sprite Sheet Frames Editor"))
                .build_window(ctx),
            container,
            editor,
            ok,
            cancel,
            width,
            height,
            grid,
            preview_container,
            cells,
        };

        ctx.add_node(UiNode::new(editor))
    }
}