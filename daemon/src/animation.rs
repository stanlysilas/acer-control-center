pub trait HardwareAnimationBackend: Send + Sync {
    fn start_breathing(&self) -> Result<(), String>;
    fn stop_animation(&self) -> Result<(), String>;
    fn breathing_selected(&self) -> bool {
        false
    }
}
