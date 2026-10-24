use std::sync::Arc;

use zbus::interface;

use crate::service::AcerService;

pub struct AcerControlDBus {
    service: Arc<AcerService>,
}

impl AcerControlDBus {
    pub fn new(service: Arc<AcerService>) -> Self {
        Self { service }
    }
}

#[interface(name = "io.github.stanlysilas.AcerControl")]
impl AcerControlDBus {
    async fn ping(&self) -> String {
        "pong".into()
    }

    async fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    async fn get_keyboard_brightness(
        &self,
    ) -> zbus::fdo::Result<u8> {
        self.service
            .get_keyboard_brightness()
            .map_err(zbus::fdo::Error::Failed)
    }

    async fn set_keyboard_brightness(
        &self,
        value: u8,
    ) -> zbus::fdo::Result<()> {
        self.service
            .set_keyboard_brightness(value)
            .map_err(zbus::fdo::Error::Failed)
    }

    async fn set_keyboard_brightness_transient(
        &self,
        value: u8,
    ) -> zbus::fdo::Result<()> {
        self.service
            .set_keyboard_brightness_transient(value)
            .map_err(zbus::fdo::Error::Failed)
    }

    async fn get_keyboard_color(
        &self,
    ) -> zbus::fdo::Result<(u8, u8, u8)> {
        self.service
            .get_keyboard_color()
            .map_err(zbus::fdo::Error::Failed)
    }

    async fn set_keyboard_color(
        &self,
        r: u8,
        g: u8,
        b: u8,
    ) -> zbus::fdo::Result<()> {
        self.service
            .set_keyboard_color(r, g, b)
            .map_err(zbus::fdo::Error::Failed)
    }

    async fn start_breathing(&self) -> zbus::fdo::Result<()> {
        self.service
            .start_breathing()
            .map_err(zbus::fdo::Error::Failed)
    }

    async fn stop_animation(&self) -> zbus::fdo::Result<()> {
        self.service
            .stop_animation()
            .map_err(zbus::fdo::Error::Failed)
    }

    async fn set_fan_max(&self) -> zbus::fdo::Result<()> {
        self.service
            .set_fan_max()
            .map_err(zbus::fdo::Error::Failed)
    }

    async fn set_fan_auto(&self) -> zbus::fdo::Result<()> {
        self.service
            .set_fan_auto()
            .map_err(zbus::fdo::Error::Failed)
    }

    async fn get_fan_max_selected(&self) -> bool {
        self.service.fan_max_selected()
    }

    async fn get_breathing_selected(&self) -> bool {
        self.service.breathing_selected()
    }
}
