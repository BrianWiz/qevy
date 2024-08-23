use crate::components::*;
use bevy::prelude::*;

#[cfg(feature = "rapier")]
use bevy_rapier3d::prelude::*;

#[cfg(feature = "rapier")]
pub fn rapier_trigger_system(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    trigger_once: Query<(Entity, &TriggerOnce), Without<TriggeredOnce>>,
    trigger_multiple: Query<(Entity, &TriggerMultiple)>,
    trigger_instigators: Query<Entity, (With<TriggerInstigator>,)>,
    mut trigger_events: EventWriter<TriggeredEvent>,
) {
    for instigator_entity in trigger_instigators.iter() {
        for (trigger_entity, trigger) in trigger_multiple.iter() {
            if rapier_context.intersection_pair(instigator_entity, trigger_entity) == Some(true) {
                trigger_events.send(TriggeredEvent {
                    target: trigger.target.clone(),
                    triggered_by: instigator_entity,
                });
            }
        }

        for (trigger_entity, trigger) in trigger_once.iter() {
            if rapier_context.intersection_pair(instigator_entity, trigger_entity) == Some(true) {
                trigger_events.send(TriggeredEvent {
                    target: trigger.target.clone(),
                    triggered_by: instigator_entity,
                });
                commands.entity(trigger_entity).insert(TriggeredOnce);
            }
        }
    }
}

#[cfg(feature = "avian")]
use avian3d::prelude::*;

#[cfg(feature = "avian")]
use bevy::utils::HashSet;

#[cfg(feature = "avian")]
pub fn avian_trigger_system(
    spatial_query: SpatialQuery,
    mut commands: Commands,
    map_entity: Query<Entity, With<Map>>,
    trigger_once: Query<
        (
            Entity,
            &TriggerOnce,
            &GlobalTransform,
            &Transform,
            &avian3d::prelude::Collider,
        ),
        Without<TriggeredOnce>,
    >,
    trigger_multiple: Query<(
        Entity,
        &TriggerMultiple,
        &GlobalTransform,
        &Transform,
        &avian3d::prelude::Collider,
    )>,
    trigger_instigators: Query<Entity, With<TriggerInstigator>>,
    mut trigger_events: EventWriter<TriggeredEvent>,
) {
    let map_entity = map_entity.get_single();

    if let Ok(map_entity) = map_entity {
        for instigator_entity in trigger_instigators.iter() {
            for (trigger_entity, trigger, gtransform, transform, collider) in
                trigger_multiple.iter()
            {
                let excluded = HashSet::from([map_entity]);
                let intersections = spatial_query.shape_intersections(
                    collider,
                    gtransform.translation(),
                    transform.rotation,
                    SpatialQueryFilter {
                        excluded_entities: excluded,
                        ..default()
                    },
                );

                for entity in intersections.iter() {
                    if *entity == instigator_entity {
                        trigger_events.send(TriggeredEvent {
                            target: trigger.target.clone(),
                            triggered_by: instigator_entity,
                        });
                        commands.entity(trigger_entity).insert(TriggeredOnce);
                    }
                }
            }

            for (trigger_entity, trigger, gtransform, transform, collider) in trigger_once.iter() {
                let intersections = spatial_query.shape_intersections(
                    collider,
                    gtransform.translation(),
                    transform.rotation,
                    SpatialQueryFilter::default(),
                );

                for entity in intersections.iter() {
                    if *entity == instigator_entity {
                        trigger_events.send(TriggeredEvent {
                            target: trigger.target.clone(),
                            triggered_by: instigator_entity,
                        });
                        commands.entity(trigger_entity).insert(TriggeredOnce);
                    }
                }
            }
        }
    }
}
