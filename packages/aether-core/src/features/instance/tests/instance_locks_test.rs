//! [`InstanceLocks`]: one change to an instance's content at a time, and no launch during one.

use crate::features::instance::{InstanceId, InstanceLocks};

fn id(id: &str) -> InstanceId {
    InstanceId::try_from(id).unwrap()
}

#[tokio::test]
async fn should_refuse_a_launch_lock_when_a_plan_holds_the_instance() {
    let locks = InstanceLocks::new();
    let held = locks.lock(&id("instance")).await;

    assert!(locks.try_lock(&id("instance")).is_none());

    drop(held);
    assert!(locks.try_lock(&id("instance")).is_some());
}

#[tokio::test]
async fn should_not_block_another_instance_when_one_is_locked() {
    let locks = InstanceLocks::new();
    let _held = locks.lock(&id("instance")).await;

    assert!(locks.try_lock(&id("other")).is_some());
}

#[tokio::test]
async fn should_let_the_second_plan_through_when_the_first_is_done() {
    let locks = InstanceLocks::new();
    let instance = id("instance");
    let first = locks.lock(&instance).await;

    let mut second = std::pin::pin!(locks.lock(&instance));
    assert!(futures::poll!(&mut second).is_pending());

    drop(first);
    let _second = second.await;
    assert!(locks.try_lock(&id("instance")).is_none());
}

#[tokio::test]
async fn should_give_up_the_wait_when_the_lock_is_held_past_it() {
    let locks = InstanceLocks::new();
    let held = locks.lock(&id("instance")).await;

    let waited = locks
        .lock_within(&id("instance"), std::time::Duration::from_millis(20))
        .await;
    assert!(waited.is_none());

    drop(held);
    let free = locks
        .lock_within(&id("instance"), std::time::Duration::from_millis(20))
        .await;
    assert!(free.is_some());
}
