use std::time::Duration;

use chrono::Utc;

use super::*;
use crate::features::{
    events::WarningEvent,
    process::{
        ProcessError, ProcessState,
        app::use_cases::fakes::{Look, World, earlier_launcher, eventually, launcher, record_of},
    },
};

/// Short enough for tests, in the order the real pace has them.
fn pace() -> Pace {
    Pace {
        owner_wait: Duration::from_millis(200),
        owner_poll: Duration::from_millis(10),
        retry_min: Duration::from_millis(10),
        retry_max: Duration::from_millis(20),
        foreign_recheck: Duration::from_millis(20),
    }
}

fn use_case(world: &World) -> AdoptGameProcessesUseCase {
    AdoptGameProcessesUseCase::new(
        world.storage.clone(),
        world.records.clone(),
        world.probe.clone(),
        world.manage.clone(),
        world.emitter.clone(),
        world.instances.clone(),
        world.game_logs.clone(),
        world.notices.clone(),
    )
    .with_pace(pace())
}

/// Shows the saved games and starts taking them over in the background.
async fn start(world: &World) -> tokio::task::JoinHandle<()> {
    let pending = use_case(world).execute().await;
    tokio::spawn(pending)
}

async fn is_pending(world: &World, id: uuid::Uuid) -> bool {
    matches!(
        world.storage.kill(id).await,
        Err(ProcessError::NotTakenOverYet { .. })
    )
}

#[tokio::test]
async fn should_show_every_saved_game_as_running_before_any_is_taken_over() {
    let first = record_of(10, earlier_launcher());
    let second = record_of(11, earlier_launcher());
    let world = World::new(&[first.clone(), second.clone()], Some(launcher()));

    let _pending = use_case(&world).execute().await;

    let shown = world.storage.list_metadata().await.unwrap();
    assert_eq!(shown.len(), 2);
    for record in [&first, &second] {
        let metadata = shown.iter().find(|m| m.uuid() == record.id).unwrap();
        assert_eq!(metadata.origin(), ProcessOrigin::Adopted);
        assert_eq!(metadata.instance_id(), &record.instance_id);
        assert_eq!(metadata.start_time(), &record.started_at);
        assert_eq!(
            world.storage.try_wait(record.id).await.unwrap(),
            Some(ProcessState::Running)
        );
        assert!(is_pending(&world, record.id).await);
    }
    assert!(world.followed().is_empty());
    assert!(world.process_events().is_empty());
}

#[tokio::test]
async fn should_take_over_the_game_when_it_runs_and_its_launcher_is_gone() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    world.probe.set(10, Look::Running);

    let resolved = start(&world).await;
    tokio::time::timeout(Duration::from_secs(10), resolved)
        .await
        .expect("the game was not settled")
        .unwrap();

    assert!(
        world
            .records
            .get(record.id)
            .unwrap()
            .owner
            .matches(&launcher())
    );
    eventually("the game is not followed", || !world.followed().is_empty()).await;
    let followed = world.followed();
    assert_eq!(followed[0].process_id, record.id);
    assert_eq!(followed[0].post_exit_command, "backup.sh");
    assert!(!followed[0].captured);
    assert_eq!(followed[0].origin, ProcessOrigin::Adopted);
    let events = world.process_events();
    assert!(
        matches!(
            events.as_slice(),
            [event] if matches!(event.event, ProcessEventType::Adopted)
                && event.adopted
                && event.process_id == record.id
        ),
        "{events:?}"
    );
    // Stopping it reaches the game now.
    world.storage.kill(record.id).await.unwrap();
    assert!(world.storage.was_killed(record.id).await);
}

#[tokio::test]
async fn should_credit_the_time_the_game_ran_unfollowed_once_the_mark_is_moved() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    world.probe.set(10, Look::Running);

    start(&world).await.await.unwrap();

    assert_eq!(*world.records.touches.lock().unwrap(), [record.id]);
    let mark = world
        .records
        .get(record.id)
        .unwrap()
        .playtime_credited_until;
    let credited = mark.signed_duration_since(record.playtime_credited_until);
    assert_eq!(
        world.instances.time_played(),
        credited.num_seconds().cast_unsigned()
    );
    assert!(
        world.instances.time_played() >= 600,
        "{}",
        world.instances.time_played()
    );
}

#[tokio::test]
async fn should_leave_the_game_to_a_launcher_that_took_it_over_after_the_records_were_read() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    world.probe.set(10, Look::Running);
    let other = OsProcessIdentity::new(3, 70, "boot".to_owned());
    world.probe.set(3, Look::Running);
    let resolution = use_case(&world).execute().await;
    // Another launcher on the same data takes the game over after this one read the record.
    world.records.set_owner(record.id, other.clone());

    let resolving = tokio::spawn(resolution);
    eventually("the other launcher was not looked at again", || {
        world.probe.times_asked(3) >= 2
    })
    .await;

    assert!(world.records.get(record.id).unwrap().owner.matches(&other));
    assert!(world.followed().is_empty());
    assert!(world.process_events().is_empty());
    assert!(is_pending(&world, record.id).await);
    assert_eq!(world.instances.time_played(), 0);

    // Once that launcher is gone, the game is this one's to take over.
    world.probe.set(3, Look::Gone);
    tokio::time::timeout(Duration::from_secs(10), resolving)
        .await
        .expect("the game was not taken over")
        .unwrap();
    assert!(
        world
            .records
            .get(record.id)
            .unwrap()
            .owner
            .matches(&launcher())
    );
    eventually("the game is not followed", || world.followed().len() == 1).await;
}

#[tokio::test]
async fn should_wait_for_the_launcher_that_followed_the_game_to_exit() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    world.probe.set(10, Look::Running);
    world.probe.set(2, Look::Running);

    let resolving = start(&world).await;
    eventually("the earlier launcher was not waited for", || {
        world.probe.times_asked(2) >= 3
    })
    .await;
    world.probe.set(2, Look::Gone);
    tokio::time::timeout(Duration::from_secs(10), resolving)
        .await
        .expect("the game was not taken over")
        .unwrap();

    assert!(
        world
            .records
            .get(record.id)
            .unwrap()
            .owner
            .matches(&launcher())
    );
    // Within the first wait: the game was looked at only once the launcher was gone.
    assert_eq!(world.probe.times_asked(10), 1);
}

#[tokio::test]
async fn should_keep_showing_the_game_while_the_launcher_that_follows_it_runs() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    world.probe.set(10, Look::Running);
    world.probe.set(2, Look::Running);

    let resolving = start(&world).await;
    // Past the wait, and looked at again later.
    let polls_in_the_wait =
        (pace().owner_wait.as_millis() / pace().owner_poll.as_millis()) as usize;
    eventually("the earlier launcher was not looked at again", || {
        world.probe.times_asked(2) > polls_in_the_wait + 2
    })
    .await;

    assert_eq!(world.probe.times_asked(10), 0);
    assert!(
        world
            .records
            .get(record.id)
            .unwrap()
            .owner
            .matches(&earlier_launcher())
    );
    assert!(is_pending(&world, record.id).await);
    assert!(world.followed().is_empty());

    world.probe.set(2, Look::Gone);
    tokio::time::timeout(Duration::from_secs(10), resolving)
        .await
        .expect("the game was not taken over")
        .unwrap();
    assert!(
        world
            .records
            .get(record.id)
            .unwrap()
            .owner
            .matches(&launcher())
    );
}

#[tokio::test]
async fn should_stop_showing_the_game_when_the_launcher_that_follows_it_ends_its_run() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    world.probe.set(2, Look::Running);

    let resolving = start(&world).await;
    eventually("the earlier launcher was not looked at", || {
        world.probe.times_asked(2) >= 1
    })
    .await;
    // That launcher saw its game end and removed the record.
    world.records.saved.lock().unwrap().clear();
    tokio::time::timeout(Duration::from_secs(10), resolving)
        .await
        .expect("the game was not settled")
        .unwrap();

    assert!(world.storage.list_metadata().await.unwrap().is_empty());
    assert!(world.notices.execute().is_empty());
    assert_eq!(world.instances.time_played(), 0);
}

#[tokio::test]
async fn should_forget_the_game_and_credit_up_to_its_last_log_write_when_it_is_gone() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    *world.game_logs.0.lock().unwrap() =
        Some(record.playtime_credited_until + chrono::Duration::seconds(300));

    start(&world).await.await.unwrap();

    assert!(world.records.get(record.id).is_none());
    assert!(world.storage.list_metadata().await.unwrap().is_empty());
    assert_eq!(world.instances.time_played(), 300);
    // The window may have shown it as running.
    let events = world.process_events();
    assert!(
        matches!(
            events.as_slice(),
            [event] if matches!(event.event, ProcessEventType::Finished)
                && event.adopted
                && !event.crashed
                && event.exit_code.is_none()
        ),
        "{events:?}"
    );
    assert!(world.followed().is_empty());
    let notices = world.notices.execute();
    assert!(
        matches!(
            notices.as_slice(),
            [WarningEvent::PostExitHookSkipped { instance_name, .. }] if instance_name == "Test Instance"
        ),
        "{notices:?}"
    );
}

#[tokio::test]
async fn should_credit_nothing_for_the_gone_game_when_a_later_run_of_the_instance_is_saved() {
    let record = record_of(10, earlier_launcher());
    let mut later = record_of(11, launcher());
    later.started_at = Utc::now();
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    *world.game_logs.0.lock().unwrap() =
        Some(record.playtime_credited_until + chrono::Duration::seconds(300));

    let pending = use_case(&world).execute().await;
    // Saved by another launcher on the same data, after this one read the records.
    world.records.saved.lock().unwrap().push(later);
    pending.await;

    assert!(world.records.get(record.id).is_none());
    assert_eq!(world.instances.time_played(), 0);
}

#[tokio::test]
async fn should_credit_nothing_for_the_gone_game_when_a_later_run_of_the_instance_is_followed() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    *world.game_logs.0.lock().unwrap() =
        Some(record.playtime_credited_until + chrono::Duration::seconds(300));

    let pending = use_case(&world).execute().await;
    world
        .storage
        .insert(
            MinecraftProcessMetadata::new(record.instance_id.clone()),
            ProcessHandle::Pending,
        )
        .await
        .unwrap();
    pending.await;

    assert!(world.records.get(record.id).is_none());
    assert_eq!(world.instances.time_played(), 0);
}

#[tokio::test]
async fn should_neither_credit_nor_tell_when_another_launcher_forgot_the_gone_game_first() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    *world.game_logs.0.lock().unwrap() =
        Some(record.playtime_credited_until + chrono::Duration::seconds(300));

    let pending = use_case(&world).execute().await;
    world.records.saved.lock().unwrap().clear();
    pending.await;

    assert_eq!(world.instances.time_played(), 0);
    assert!(world.notices.execute().is_empty());
    assert!(world.storage.list_metadata().await.unwrap().is_empty());
}

#[tokio::test]
async fn should_tell_nothing_of_a_hook_when_the_gone_game_had_none() {
    let mut record = record_of(10, earlier_launcher());
    record.post_exit_command = String::new();
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));

    start(&world).await.await.unwrap();

    assert!(world.records.get(record.id).is_none());
    assert!(world.notices.execute().is_empty());
}

#[tokio::test]
async fn should_keep_the_game_and_try_again_when_the_os_cannot_tell_whether_it_runs() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    world.probe.set(10, Look::Unknown);

    let resolving = start(&world).await;
    eventually("the game was not looked at again", || {
        world.probe.times_asked(10) >= 3
    })
    .await;

    assert!(world.records.get(record.id).is_some());
    assert!(is_pending(&world, record.id).await);
    assert!(world.followed().is_empty());

    world.probe.set(10, Look::Running);
    tokio::time::timeout(Duration::from_secs(10), resolving)
        .await
        .expect("the game was not taken over")
        .unwrap();
    assert!(
        world
            .records
            .get(record.id)
            .unwrap()
            .owner
            .matches(&launcher())
    );
}

#[tokio::test]
async fn should_settle_the_other_games_when_one_cannot_be_looked_at() {
    let unknown = record_of(10, earlier_launcher());
    let running = record_of(11, earlier_launcher());
    let gone = record_of(12, earlier_launcher());
    let world = World::new(
        &[unknown.clone(), running.clone(), gone.clone()],
        Some(launcher()),
    );
    world.probe.set(10, Look::Unknown);
    world.probe.set(11, Look::Running);

    let _resolving = start(&world).await;
    eventually("the running game was not followed", || {
        world.followed().len() == 1
    })
    .await;
    eventually("the gone game was not forgotten", || {
        world.records.get(gone.id).is_none()
    })
    .await;

    assert_eq!(world.followed()[0].process_id, running.id);
    assert!(world.records.get(unknown.id).is_some());
    assert!(is_pending(&world, unknown.id).await);
}

#[tokio::test]
async fn should_keep_showing_the_game_when_the_launcher_does_not_know_its_own_process() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), None);
    world.probe.set(10, Look::Running);

    let _resolving = start(&world).await;
    eventually("the game was not looked at again", || {
        world.probe.times_asked(10) >= 2
    })
    .await;

    assert!(
        world
            .records
            .get(record.id)
            .unwrap()
            .owner
            .matches(&earlier_launcher())
    );
    assert!(is_pending(&world, record.id).await);
    assert!(world.followed().is_empty());
}

#[tokio::test]
async fn should_show_nothing_when_the_saved_games_cannot_be_read() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    *world.records.failing_reads.lock().unwrap() = true;

    use_case(&world).execute().await.await;

    assert!(world.storage.list_metadata().await.unwrap().is_empty());
    assert!(world.records.get(record.id).is_some());
}

#[tokio::test]
async fn should_leave_the_game_to_the_launcher_that_took_it_over_first() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    world.probe.set(10, Look::Running);
    let other = OsProcessIdentity::new(3, 70, "boot".to_owned());
    world.probe.set(3, Look::Running);
    // Between this launcher's last look at the record and its take over.
    *world.records.taken_first_by.lock().unwrap() = Some(other.clone());

    let resolving = start(&world).await;
    eventually("the other launcher was not looked at", || {
        world.probe.times_asked(3) >= 1
    })
    .await;

    assert!(world.records.get(record.id).unwrap().owner.matches(&other));
    assert!(world.followed().is_empty());
    assert!(world.process_events().is_empty());
    assert!(is_pending(&world, record.id).await);
    assert_eq!(world.instances.time_played(), 0);
    resolving.abort();
}

#[tokio::test]
async fn should_credit_from_the_mark_the_earlier_launcher_moved_while_it_was_waited_for() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    world.probe.set(10, Look::Running);
    world.probe.set(2, Look::Running);

    let resolving = start(&world).await;
    eventually("the earlier launcher was not waited for", || {
        world.probe.times_asked(2) >= 2
    })
    .await;
    // Its last checkpoint before it quits credits up to now.
    world
        .records
        .touch_playtime(record.id, Utc::now())
        .await
        .unwrap();
    world.probe.set(2, Look::Gone);
    tokio::time::timeout(Duration::from_secs(10), resolving)
        .await
        .expect("the game was not taken over")
        .unwrap();

    assert!(
        world
            .records
            .get(record.id)
            .unwrap()
            .owner
            .matches(&launcher())
    );
    assert!(
        world.instances.time_played() < 60,
        "credited again: {}",
        world.instances.time_played()
    );
}

#[tokio::test]
async fn should_not_take_over_the_game_when_it_is_no_longer_shown() {
    let record = record_of(10, earlier_launcher());
    let world = World::new(std::slice::from_ref(&record), Some(launcher()));
    world.probe.set(10, Look::Running);

    let pending = use_case(&world).execute().await;
    world.storage.remove(record.id).await.unwrap();
    pending.await;

    assert!(
        world
            .records
            .get(record.id)
            .unwrap()
            .owner
            .matches(&earlier_launcher())
    );
    assert!(world.followed().is_empty());
    assert_eq!(world.instances.time_played(), 0);
}
