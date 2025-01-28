pub mod anr_result_bean;
pub mod bugreport;
pub mod lock_bean;
pub mod log_item_bean;
pub mod result_item_bean;

pub trait Analyse {
    fn analyse(
        &self,
        file: &std::path::Path,
        param_string: &str,
    ) -> Vec<result_item_bean::ResultItemBean>;
}
