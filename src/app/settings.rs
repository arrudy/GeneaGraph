#[allow(dead_code)]

pub struct SettingsInteraction {
    pub dragging_enabled: bool,
    pub node_clicking_enabled: bool,
    pub node_selection_enabled: bool,
    pub node_selection_multi_enabled: bool,
    pub edge_clicking_enabled: bool,
    pub edge_selection_enabled: bool,
    pub edge_selection_multi_enabled: bool,
}


impl Default for SettingsInteraction{
    fn default() -> Self {
        Self { dragging_enabled: true, 
            node_clicking_enabled: true, 
            node_selection_enabled: true, 
            node_selection_multi_enabled: false, 
            edge_clicking_enabled: true, 
            edge_selection_enabled: true, 
            edge_selection_multi_enabled: false 
        }
    }

}


pub struct SettingsNavigation {
    pub fit_to_screen_enabled: bool,
    pub zoom_and_pan_enabled: bool,
    pub screen_padding: f32,
    pub zoom_speed: f32,
}

impl Default for SettingsNavigation {
    fn default() -> Self {
        Self {
            screen_padding: 0.3,
            zoom_speed: 0.02,
            fit_to_screen_enabled: false,
            zoom_and_pan_enabled: true,
        }
    }
}


pub struct SettingsStyle {
    pub labels_always: bool,
}


impl Default for SettingsStyle {
    fn default() -> Self {
        Self {
            labels_always : true
        }
    }
}