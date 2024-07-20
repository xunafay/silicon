use std::any::TypeId;

use bevy::{
    asset::{ReflectAsset, UntypedAssetId},
    log::info,
    prelude::{
        AppTypeRegistry, Entity, Mut, ReflectResource, Resource, SystemParamFunction, With, World,
    },
    reflect::TypeRegistry,
    render::camera::{Camera, Projection},
    transform::components::GlobalTransform,
};
use bevy_egui::egui::{self};
use bevy_inspector_egui::bevy_inspector::{
    self,
    hierarchy::{hierarchy_ui, SelectedEntities},
    ui_for_entities_shared_components, ui_for_entity_with_children,
};
use bevy_math::Mat4;
use bevy_trait_query::One;
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use egui_plot::{Corner, Legend, Line, Plot, VLine};
use silicon_core::{Clock, Neuron, SpikeRecorder, ValueRecorder};
use simulator::{PruneSettings, SimpleSpikeRecorder};
use synapses::{Synapse, SynapseType};
use transform_gizmo_egui::{Color32, GizmoMode};

use crate::{structure::feed_forward::FeedForwardNetwork, EncoderState, Insights};

use super::SimulationUiState;

#[derive(Eq, PartialEq)]
pub enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

#[derive(Resource)]
pub struct UiState {
    pub state: DockState<EguiWindow>,
    pub viewport_rect: egui::Rect,
    pub selected_entities: SelectedEntities,
    pub selection: InspectorSelection,
    pub gizmo_mode: GizmoMode,
}

impl UiState {
    pub fn new() -> Self {
        let mut state = DockState::new(vec![EguiWindow::GameView]);
        let tree = state.main_surface_mut();
        // let [game, _inspector] =
        //     tree.split_right(NodeIndex::root(), 0.75, vec![EguiWindow::Inspector]);
        let [game, _bottom] =
            tree.split_below(NodeIndex::root(), 0.8, vec![EguiWindow::GraphViewer]);
        let [_game, _hierarchy] = tree.split_right(
            game,
            0.75,
            vec![
                EguiWindow::SimulationSettings,
                EguiWindow::Training,
                EguiWindow::NeuronInspector,
            ],
        );

        Self {
            state,
            selected_entities: SelectedEntities::default(),
            selection: InspectorSelection::Entities,
            viewport_rect: egui::Rect::NOTHING,
            gizmo_mode: GizmoMode::TranslateXY,
        }
    }

    pub fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
            gizmo_mode: self.gizmo_mode,
        };
        DockArea::new(&mut self.state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

#[derive(Debug)]
pub enum EguiWindow {
    GameView,
    Hierarchy,
    Resources,
    Assets,
    Inspector,
    GraphViewer,
    SimulationSettings,
    NeuronInspector,
    Training,
}
struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut SelectedEntities,
    selection: &'a mut InspectorSelection,
    viewport_rect: &'a mut egui::Rect,
    gizmo_mode: GizmoMode,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, window: &mut Self::Tab) {
        let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        match window {
            EguiWindow::GameView => {
                *self.viewport_rect = ui.clip_rect();

                draw_gizmo(ui, self.world, self.selected_entities, self.gizmo_mode);
            }
            EguiWindow::Hierarchy => {
                let selected = hierarchy_ui(self.world, ui, self.selected_entities);
                if selected {
                    *self.selection = InspectorSelection::Entities;
                }
            }
            EguiWindow::Resources => select_resource(ui, &type_registry, self.selection),
            EguiWindow::Assets => select_asset(ui, &type_registry, self.world, self.selection),
            EguiWindow::Inspector => match *self.selection {
                InspectorSelection::Entities => match self.selected_entities.as_slice() {
                    &[entity] => ui_for_entity_with_children(self.world, entity, ui),
                    entities => ui_for_entities_shared_components(self.world, entities, ui),
                },
                InspectorSelection::Resource(type_id, ref name) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_resource(
                        self.world,
                        type_id,
                        ui,
                        name,
                        &type_registry,
                    )
                }
                InspectorSelection::Asset(type_id, ref name, handle) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_asset(
                        self.world,
                        type_id,
                        handle,
                        ui,
                        &type_registry,
                    );
                }
            },
            EguiWindow::GraphViewer => {
                ui.label("Neuron Inspector");
                plotter(ui, self.world);
            }
            EguiWindow::SimulationSettings => {
                ui.label("Simulation Settings");
                simulation_settings(ui, self.world);
            }
            EguiWindow::Training => {
                ui.label("Training settings");
                training_settings(ui, self.world);
            }
            EguiWindow::NeuronInspector => {
                let selected = {
                    let insights = self.world.get_resource::<Insights>().unwrap();
                    insights.selected_entity.clone()
                };

                if let Some(selected) = selected {
                    bevy_inspector::ui_for_entity(self.world, selected, ui);
                    ui.separator();
                    let outgoing_synapses = self
                        .world
                        .query::<(Entity, One<&dyn Synapse>)>()
                        .iter(self.world)
                        .filter_map(|(entity, synapse)| {
                            if synapse.get_presynaptic() == selected {
                                Some(entity)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    let incoming_synapses = self
                        .world
                        .query::<(Entity, One<&dyn Synapse>)>()
                        .iter(self.world)
                        .filter_map(|(entity, synapse)| {
                            if synapse.get_postsynaptic() == selected {
                                Some(entity)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    ui.label("Outgoing synapses");
                    for entity in outgoing_synapses {
                        bevy_inspector::ui_for_entity(self.world, entity, ui);
                    }
                    ui.separator();
                    ui.label("Incoming synapses");
                    for entity in incoming_synapses {
                        bevy_inspector::ui_for_entity(self.world, entity, ui);
                    }
                } else {
                    ui.label("No neuron selected");
                }
            }
        }
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        format!("{window:?}").into()
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

fn training_settings(ui: &mut egui::Ui, world: &mut World) {
    bevy_inspector::ui_for_resource::<EncoderState>(world, ui);
}

fn simulation_settings(ui: &mut egui::Ui, world: &mut World) {
    world.resource_scope(|world, mut clock: Mut<Clock>| {
        ui.label(format!("Simulated time: {:.2}ms", clock.time));

        world.resource_scope(|_, mut state: Mut<SimulationUiState>| {
            ui.add(
                egui::Slider::new(&mut state.simulation_time_slider, 0.0..=100.0)
                    .clamp_to_range(false)
                    .text("Time to simulate in ms"),
            );
            ui.add(
                egui::Slider::new(&mut clock.tau, 0.001..=0.1)
                    .clamp_to_range(false)
                    .text("Time constant in ms"),
            );

            ui.add(egui::Checkbox::new(
                &mut clock.run_indefinitely,
                "Run indefinitely",
            ))
            .on_hover_text("Run the simulation indefinitely");

            let button = ui
                .button("Run")
                .on_hover_text("Run the simulation for the specified time");
            if button.clicked() {
                clock.time_to_simulate = state.simulation_time_slider;
                info!("Running simulation for {} ms", state.simulation_time_slider);
            }
        })
    });

    ui.separator();

    ui.label("Pruning settings");
    ui.add(
        egui::Slider::new(
            &mut world
                .get_resource_mut::<PruneSettings>()
                .unwrap()
                .min_weight,
            0.0..=1.0,
        )
        .clamp_to_range(false)
        .text("Minimum weight to prune"),
    );

    ui.separator();

    ui.label("Reconnect");
    let button = ui
        .button("Reconnect neurons")
        .on_hover_text("Reconnect neurons");
    if button.clicked() {
        info!("Reconnecting neurons");

        let mut new_synapses: Vec<(Entity, Entity, SynapseType)> = vec![];

        let mut neurons = world.query::<(Entity, One<&dyn Neuron>)>();
        let mut synapses = world.query::<(Entity, One<&dyn Synapse>)>();

        for [(pre_synaptic, _), (post_synaptic, _)] in neurons.iter_combinations(world) {
            if pre_synaptic == post_synaptic {
                return;
            }

            let synapse = synapses
                .iter(world)
                .find(|(_, synapse)| {
                    synapse.get_presynaptic() == pre_synaptic
                        && synapse.get_postsynaptic() == post_synaptic
                })
                .map(|(entity, _)| entity);

            if synapse.is_none() {
                if rand::random::<f64>() < 0.8 {
                    continue;
                }

                info!(
                    "Reconnecting neurons {:?} and {:?}",
                    pre_synaptic, post_synaptic
                );

                let synapse_type = if rand::random::<f64>() < 0.8 {
                    SynapseType::Excitatory
                } else {
                    SynapseType::Inhibitory
                };

                new_synapses.push((pre_synaptic, post_synaptic, synapse_type));
            }
        }

        for synapse in new_synapses {
            info!(
                "Creating synapse between {:?} and {:?}",
                synapse.0, synapse.1
            );
            FeedForwardNetwork::create_synapse(
                &synapse.0,
                &synapse.1,
                synapse.2,
                (0.1, 0.3),
                world,
            );
        }
    }

    ui.separator();

    ui.label(format!(
        "Total neurons: {}",
        world.query::<One<&dyn Neuron>>().iter(world).count(),
    ));

    ui.label(format!(
        "Total synapses: {}",
        world.query::<One<&dyn Synapse>>().iter(world).count(),
    ));

    ui.label(format!(
        "Average synapse weight: {:.2}",
        world
            .query::<One<&dyn Synapse>>()
            .iter(world)
            .map(|synapse| synapse.get_weight())
            .sum::<f64>()
            / world.query::<One<&dyn Synapse>>().iter(world).count() as f64
    ));
}

#[derive(Debug, Default, Resource)]
pub struct PlotterConfig {
    pub window_size: usize,
}

fn plotter(ui: &mut egui::Ui, world: &mut World) {
    let mut membrane_plotters = world.query::<(Entity, &ValueRecorder, &SimpleSpikeRecorder)>();
    let mut synapse_plotters = world.query::<(Entity, &ValueRecorder, One<&dyn Synapse>)>();
    let insights = world.get_resource::<Insights>().unwrap();
    let clock = world.get_resource::<Clock>().unwrap();
    let config = world.get_resource::<PlotterConfig>().unwrap();

    let selected_membrane_plotter = membrane_plotters.iter(world).find(|(entity, _, _)| {
        insights
            .selected_entity
            .map_or(false, |selected_entity| *entity == selected_entity)
    });

    let synapse_plots: Vec<_> = synapse_plotters
        .iter(world)
        .filter(|(_, _, synapse)| {
            insights.selected_entity.map_or(false, |selected_entity| {
                synapse.get_presynaptic() == selected_entity
                    || synapse.get_postsynaptic() == selected_entity
            })
        })
        .collect();

    if insights.selected_entity.is_none() {
        ui.label("No neuron selected");
        return;
    }

    if let Some((entity, plotter, spikes)) = selected_membrane_plotter {
        let plot = Plot::new("Neuron")
            .legend(Legend::default().position(Corner::LeftBottom))
            .height(200.0);
        plot.show(ui, |plot_ui| {
            let spikes = spikes
                .get_spikes()
                .iter()
                .filter(|time| **time >= clock.time - config.window_size as f64)
                .copied()
                .collect::<Vec<_>>();
            for spike in spikes {
                plot_ui.vline(VLine::new(spike).color(Color32::RED));
            }

            let points: Vec<[f64; 2]> = plotter
                .values
                .iter()
                .filter(|(time, _)| *time >= clock.time - config.window_size as f64)
                .map(|(time, value)| [*time, *value])
                .collect();

            plot_ui.line(
                Line::new(points)
                    .name(format!("{:?}", entity))
                    .color(Color32::BLUE),
            );
        });
    }

    let plot = Plot::new("Synapses")
        .legend(Legend::default().position(Corner::LeftBottom))
        .height(200.0);
    plot.show(ui, |plot_ui| {
        for (entity, plotter, synapse) in synapse_plots.iter() {
            let points: Vec<[f64; 2]> = plotter
                .values
                .iter()
                .filter(|(time, _)| *time >= clock.time - config.window_size as f64)
                .map(|(time, value)| [*time, *value])
                .collect();

            plot_ui.line(Line::new(points).name(format!("{:?}", entity)).color(
                match synapse.get_type() {
                    SynapseType::Excitatory => Color32::BLUE,
                    SynapseType::Inhibitory => Color32::RED,
                },
            ));
        }
    });
}

fn select_resource(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    selection: &mut InspectorSelection,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    for (resource_name, type_id) in resources {
        let selected = match *selection {
            InspectorSelection::Resource(selected, _) => selected == type_id,
            _ => false,
        };

        if ui.selectable_label(selected, resource_name).clicked() {
            *selection = InspectorSelection::Resource(type_id, resource_name.to_string());
        }
    }
}

fn select_asset(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    world: &World,
    selection: &mut InspectorSelection,
) {
    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
                reflect_asset,
            ))
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    for (asset_name, asset_type_id, reflect_asset) in assets {
        let handles: Vec<_> = reflect_asset.ids(world).collect();

        ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
            for handle in handles {
                let selected = match *selection {
                    InspectorSelection::Asset(_, _, selected_id) => selected_id == handle,
                    _ => false,
                };

                if ui
                    .selectable_label(selected, format!("{:?}", handle))
                    .clicked()
                {
                    *selection =
                        InspectorSelection::Asset(asset_type_id, asset_name.to_string(), handle);
                }
            }
        });
    }
}

#[allow(unused, clippy::needless_return)]
fn draw_gizmo(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_entities: &SelectedEntities,
    gizmo_mode: GizmoMode,
) {
    let (cam_transform, projection) = world
        .query_filtered::<(&GlobalTransform, &Projection), With<Camera>>()
        .single(world);
    let view_matrix = Mat4::from(cam_transform.affine().inverse());
    // let projection_matrix = projection.get_projection_matrix();

    if selected_entities.len() != 1 {
        return;
    }

    // for selected in selected_entities.iter() {
    //     let Some(transform) = world.get::<Transform>(selected) else {
    //         continue;
    //     };
    //     let model_matrix = transform.compute_matrix();

    //     let mut gizmo = transform_gizmo_egui::Gizmo::new(GizmoConfig {
    //         view_matrix: view_matrix.into(),
    //         projection_matrix: projection_matrix.into(),
    //         orientation: GizmoOrientation::Local,
    //         modes: EnumSet::from(gizmo_mode),
    //         ..Default::default()
    //     });
    //     let Some([result]) = gizmo
    //         .interact(ui, model_matrix.into())
    //         .map(|(_, res)| res.as_slice())
    //     else {
    //         continue;
    //     };

    //     let mut transform = world.get_mut::<Transform>(selected).unwrap();
    //     *transform = Transform {
    //         translation: Vec3::from(<[f64; 3]>::from(result.translation)),
    //         rotation: Quat::from_array(<[f64; 4]>::from(result.rotation)),
    //         scale: Vec3::from(<[f64; 3]>::from(result.scale)),
    //     };
    // }
}
