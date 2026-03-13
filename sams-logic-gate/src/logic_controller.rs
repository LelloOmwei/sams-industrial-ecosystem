// Factory module for creating logic controllers
use crate::common_types::LogicController;

// Factory function that works with both implementations
pub fn create_logic_controller() -> Box<dyn LogicController + Send + Sync> {
    #[cfg(feature = "closed-source")]
    {
        return crate::slc_core::create_slc();
    }
    
    #[cfg(feature = "open-source")]
    {
        return crate::mock_logic::create_slc();
    }
}
