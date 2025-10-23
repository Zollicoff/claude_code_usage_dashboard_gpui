use gpui::*;

#[derive(Clone, PartialEq)]
pub struct RunIntegration;

impl_actions!(dashboard, [RunIntegration]);

#[derive(Clone, PartialEq)]
pub struct TestConnections;

impl_actions!(dashboard, [TestConnections]);

#[derive(Clone, PartialEq)]
pub struct RefreshStats;

impl_actions!(dashboard, [RefreshStats]);
