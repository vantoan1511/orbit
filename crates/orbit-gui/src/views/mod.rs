pub mod dashboard;
pub mod details_drawer;
pub mod log_viewer;
pub mod modals;
pub mod navigation;
pub mod resource_table;
pub mod settings;
pub mod yaml_editor;
#[cfg(debug_assertions)]
pub mod showcase;

pub use dashboard::{compute_cluster_summary, ClusterSummary};
pub use details_drawer::{available_actions_for_resource_kind, ResourceAction, SelectedResourceDetails};
pub use modals::{ActiveModal, AddClusterModalMode, SearchResultItem, SearchResultKind};
pub use navigation::{
    activity_bar_view, category_sub_nav_items, extract_all_cluster_tags, filter_clusters,
    get_cluster_display_name, is_local_cluster, sub_sidebar_view, SidebarCategory, SubNavItem,
};
pub use resource_table::{columns_for_kind, TableColumn};
pub use settings::SettingsViewState;
#[cfg(debug_assertions)]
pub use showcase::{showcase_view, ShowcaseMessage, ShowcaseSection, ShowcaseState};
