use crate::components::*;
use bevy::prelude::*;

#[cfg(feature = "rapier")]
use bevy_rapier3d::prelude::*;

#[cfg(feature = "rapier")]
pub fn trigger_system(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    trigger_once: Query<(Entity, &TriggerOnce), Without<TriggeredOnce>>,
    trigger_multiple: Query<(Entity, &TriggerMultiple)>,
    trigger_instigators: Query<Entity, With<TriggerInstigator>>,
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
