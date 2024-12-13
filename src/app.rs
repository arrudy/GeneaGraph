#[allow(unused_imports)]
use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}};

use crossbeam::channel::{unbounded, Receiver, Sender};
use egui::CollapsingHeader;
use petgraph::stable_graph::StableGraph;


mod settings;
mod person;

const SERVER_ADDRESS : &str = "https://chmobl-e2are7apgsdqd4g4.polandcentral-01.azurewebsites.net";
const EVENTS_LIMIT: usize = 100;


macro_rules! pretty_string {
    ($($arg:tt)*) => {
        format!($($arg)*)
    };
}

pub struct TemplateApp {

    show_help : bool,

    c_person : person::Person,
    c_is_dead : bool,

    g_person : person::Person,

    query_result : Vec<HashMap<String,String>>,
    resultbar_s : String,

    id_src: String,
    id_tgt : String,

    g: egui_graphs::Graph<(), ()>,
    node_map : HashMap<u32, petgraph::prelude::NodeIndex>,

    settings_interaction: settings::SettingsInteraction,
    settings_navigation: settings::SettingsNavigation,
    settings_style: settings::SettingsStyle,


    last_events: Vec<String>,
    event_publisher: Sender<egui_graphs::events::Event>,
    event_consumer: Receiver<egui_graphs::events::Event>,

    task_node_refresh : Option<Task<Result<Vec<person::Person>, Box<dyn std::error::Error>>>>,
    task_conn_refresh : Option<Task<Result<Vec<person::Link>, Box<dyn std::error::Error>>>>,
    task_gperson_refresh : Option<Task<Result<person::Person, Box<dyn std::error::Error>>>>,
    task_query_result : Option<Task<Result<Vec<HashMap<String,String>>, Box<dyn std::error::Error>>>>

}

impl Default for TemplateApp {
    fn default() -> Self {

        let mut g = petgraph::stable_graph::StableGraph::new();
        g.add_node(());

        let (event_publ, event_cons) = unbounded();
        Self {
            // Example stuff:
            show_help : false,

            id_src : String::new(),
            id_tgt : String::new(),

            c_person: person::Person{name : String::new(), last_name: String::new(), 
                birth_dt: chrono::naive::NaiveDate::from_ymd_opt(2024, 12, 6).unwrap(), 
                death_dt: None,
                country: String::new(), id: None},
            c_is_dead: false,


            g_person: person::Person{name : String::new(), last_name: String::new(), 
                birth_dt: chrono::naive::NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(), 
                death_dt: None,
                country: String::new(), id: None},

            query_result : Vec::new(),
            resultbar_s : "".to_owned(),

            g: egui_graphs::Graph::from( &g).into(),
            node_map : HashMap::new(),

            settings_navigation : settings::SettingsNavigation::default(),
            settings_interaction : settings::SettingsInteraction::default(),
            settings_style: settings::SettingsStyle::default(),

            last_events: Vec::default(),
            event_consumer : event_cons ,
            event_publisher : event_publ,

            task_node_refresh: Some(Task::spawn(generic_get::<Vec<person::Person>>("/people".to_owned()))),
            task_conn_refresh: Some(Task::spawn(generic_get::<Vec<person::Link>>("/relations".to_owned()))),
            task_gperson_refresh: None,
            task_query_result : None
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        Default::default()
    }
}



impl eframe::App for TemplateApp {

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui



        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                    
                }

                egui::widgets::global_theme_preference_buttons(ui);

                if ui.button("Pomoc").clicked() {
                    // Show a modal or open a section for the help text
                    self.show_help = true; // Assume a mutable `show_help` boolean exists in your app's state
                }
            });
        });

        if self.show_help {
            egui::Window::new("Help")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Pomoc");
                    ui.separator();
                    ui.label("Sterowanie:");
                    ui.label("- LPM: wybierz węzeł / połączenie");
                    ui.label("- ŚPM (przytrzymać) przemieść graf na ekranie");
                    ui.label("- LCtrl + ŚPM (obrót) przybliż/oddal graf");
                    ui.label("- LAlt + C: Utwórz połączenie COUSIN_OF - 2 węzły");
                    ui.label("- LAlt + J: Utwórz połączenie PARENT_OF - 2 węzły");
                    ui.label("- Delete: Usuń węzeł / połączenie");
                    ui.label("- LAlt + R: Odśwież graf");
                    ui.label("");
                    ui.label("Do utworzenia połączenia wymagane jest wybranie dwóch węzłów. Kierunek połączenia jest równy \nID źródła -> ID celu (menu Debug). ");
                    ui.label("Query z zakładki Zapytania mogą wymagać wybranego min. 1 węzła. Więcej informacji znajduje się w dokumentacji.");
                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            self.show_help = false; // Hide the help window
                        }
                    });
                });
        }
        


        egui::SidePanel::right("right_panel")
            .min_width(250.)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
/*                     self.draw_section_app(ui);
                    ui.add_space(10.);
                    self.draw_section_debug(ui);
                    ui.add_space(10.);*/
                    self.draw_section_widget(ui);
                });
            });

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui|{

            
            CollapsingHeader::new("Wynik query").default_open(true).show(ui, |ui| {
                if !self.resultbar_s.is_empty()
                {
                ui.label(self.resultbar_s.clone());
                }
                draw_table(ui, &self.query_result);
            });

            ui.separator();

            /*ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });*/
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Powered by egui");


            ui.separator();

            let settings_interaction = &egui_graphs::SettingsInteraction::new()
                .with_node_selection_enabled(self.settings_interaction.node_selection_enabled)
                .with_node_selection_multi_enabled(
                    self.settings_interaction.node_selection_multi_enabled,
                )
                .with_dragging_enabled(self.settings_interaction.dragging_enabled)
                .with_node_clicking_enabled(self.settings_interaction.node_clicking_enabled)
                .with_edge_clicking_enabled(self.settings_interaction.edge_clicking_enabled)
                .with_edge_selection_enabled(self.settings_interaction.edge_selection_enabled)
                .with_edge_selection_multi_enabled(
                    self.settings_interaction.edge_selection_multi_enabled,
                );
            let settings_navigation = &egui_graphs::SettingsNavigation::new()
                .with_zoom_and_pan_enabled(self.settings_navigation.zoom_and_pan_enabled)
                .with_fit_to_screen_enabled(self.settings_navigation.fit_to_screen_enabled)
                .with_zoom_speed(self.settings_navigation.zoom_speed);
            let settings_style = &egui_graphs::SettingsStyle::new()
                .with_labels_always(self.settings_style.labels_always);

            ui.add(&mut egui_graphs::GraphView::<
                _,
                _,
                _,
                _,
                egui_graphs::DefaultNodeShape,
                egui_graphs::DefaultEdgeShape,
                //egui_graphs::LayoutStateHierarchical,
                //egui_graphs::LayoutHierarchical,
            >::new(&mut self.g)
            .with_interactions(settings_interaction)
            .with_navigations(settings_navigation).with_events(&self.event_publisher)
            .with_styles(settings_style)
        );

        
        

        

            
        });
        self.process_inputs(ctx);
        self.handle_events();
        self.process_tasks();
    }
}



impl TemplateApp

{

fn draw_section_widget(&mut self, ui: &mut egui::Ui) {
    CollapsingHeader::new("Konfiguracja")
    .show(ui, |ui| {
        CollapsingHeader::new("Ustawienia nawigacji").default_open(true).show(ui, |ui|{
            if ui
                .checkbox(&mut self.settings_navigation.fit_to_screen_enabled, "fit_to_screen")
                .changed()
                && self.settings_navigation.fit_to_screen_enabled
            {
                self.settings_navigation.zoom_and_pan_enabled = false
            };
            ui.label("Enable fit to screen to fit the graph to the screen on every frame.");

            ui.add_space(5.);

            ui.add_enabled_ui(!self.settings_navigation.fit_to_screen_enabled, |ui| {
                ui.vertical(|ui| {
                    ui.checkbox(&mut self.settings_navigation.zoom_and_pan_enabled, "zoom_and_pan");
                    ui.label("Przybliż z użyciem ctrl + obrót kółka myszy, przesuwaj z wciśniętym kółkiem myszu.");
                }).response.on_disabled_hover_text("Wyłącz fit_to_screen aby móc włączyć zoom_and_pan");
            });
        });

        CollapsingHeader::new("Style").show(ui, |ui| {
            ui.checkbox(&mut self.settings_style.labels_always, "labels_always");
            ui.label("Pokazuj identyfikatory zawsze.");
        });

        CollapsingHeader::new("Interaction").show(ui, |ui| {
            if ui.checkbox(&mut self.settings_interaction.dragging_enabled, "dragging_enabled").clicked() && self.settings_interaction.dragging_enabled {
                self.settings_interaction.node_clicking_enabled = true;
            };
            ui.label("Przesuwaj zaznaczone węzły na ekranie z użyciem myszy.");

            ui.add_space(5.);

            ui.add_enabled_ui(!(self.settings_interaction.dragging_enabled || self.settings_interaction.node_selection_enabled || self.settings_interaction.node_selection_multi_enabled), |ui| {
                ui.vertical(|ui| {
                    ui.checkbox(&mut self.settings_interaction.node_clicking_enabled, "node_clicking_enabled");
                    ui.label("Check click events in last events");
                }).response.on_disabled_hover_text("node click is enabled when any of the interaction is also enabled");
            });


            ui.add_space(5.);

            ui.add_enabled_ui(!(self.settings_interaction.edge_selection_enabled || self.settings_interaction.edge_selection_multi_enabled), |ui| {
                ui.vertical(|ui| {
                    ui.checkbox(&mut self.settings_interaction.edge_clicking_enabled, "edge_clicking_enabled");
                    ui.label("Check click events in last events");
                }).response.on_disabled_hover_text("edge click is enabled when any of the interaction is also enabled");
            });
        
        });



    });


    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //
    //                                                               DEBUG
    //
    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////


    CollapsingHeader::new("Debug").default_open(true).show(ui, |ui| {


        ui.horizontal(|ui| {
            ui.label("ID źródła");
            ui.add(egui::TextEdit::singleline(&mut self.id_src).interactive(false));
            });

        ui.horizontal(|ui| {
            ui.label("ID celu");
            ui.add(egui::TextEdit::singleline(&mut self.id_tgt).interactive(false));
            });

        CollapsingHeader::new("Last Events").show(ui, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, true]).max_height(200.).show(ui, |ui| {
                self.last_events.iter().rev().for_each(|event| {
                    ui.label(event);
                });
            });
        });
        
    });



    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //
    //                                                               DISPLAY
    //
    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////


    CollapsingHeader::new("Wyświetl").default_open(true).show(ui, |ui| {

        ui.label("Zebrane dane o węźle");
        if self.g_person.id.is_some()
        {
        ui.horizontal(|ui| {
            ui.label("Imię");
            ui.add(egui::TextEdit::singleline(&mut self.g_person.name).interactive(false));
            });
        ui.horizontal(|ui| {
            ui.label("Nazwisko");
            ui.add(egui::TextEdit::singleline(&mut self.g_person.last_name).interactive(false));
            });
        ui.horizontal(|ui| {
            ui.label("Data urodzenia");
            ui.add(egui::TextEdit::singleline(&mut self.g_person.birth_dt.to_string()).interactive(false));
            //ui.add(egui_extras::DatePickerButton::new( &mut self.g_person.birth_dt).format("%d/%m/%Y").id_salt("date_bt_get"));
            });
        if self.g_person.death_dt.is_some()
        {
            ui.horizontal(|ui| {
                ui.label("Data śmierci");
                ui.add(egui::TextEdit::singleline(&mut self.g_person.death_dt.unwrap().to_string()).interactive(false));
                });
        }
        else
        {
            ui.label("Żywy");
        }
        ui.horizontal(|ui| {
            ui.label("Kraj pochodzenia");
            ui.add(egui::TextEdit::singleline(&mut self.g_person.country).interactive(false));
            });
        }
    });

    


    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //
    //                                                               CREATE
    //
    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    CollapsingHeader::new("Utwórz").default_open(true).show(ui, |ui| {

        ui.label("Utwórz osobę");
        ui.horizontal(|ui| {
            ui.label("Imię");
            ui.add(egui::TextEdit::singleline(&mut self.c_person.name));
            });
        ui.horizontal(|ui| {
            ui.label("Nazwisko");
            ui.add(egui::TextEdit::singleline(&mut self.c_person.last_name));
            });
        ui.horizontal(|ui| {
            ui.label("Data urodzenia");
            ui.add(egui_extras::DatePickerButton::new( &mut self.c_person.birth_dt).format("%d/%m/%Y").id_salt("date_bt_create"));
            });
        if self.c_is_dead == true
        {
            if self.c_person.death_dt.is_none()
            {
                web_sys::console::log_1(&"Here comes the trouble, this should never display".into());
                self.c_person.death_dt = Some(chrono::naive::NaiveDate::from_ymd_opt(2024, 12, 6).unwrap());
            }

            ui.horizontal(|ui| {
                ui.label("Data śmierci");
                ui.add(egui_extras::DatePickerButton::new( &mut self.c_person.death_dt.as_mut().unwrap()).format("%d/%m/%Y").id_salt("date_dt_create"));
                });
        }
        else
        {
            self.c_person.death_dt = None
        }



        ui.horizontal(|ui| {
            ui.checkbox(&mut self.c_is_dead, "nie żyje");
            });
        ui.horizontal(|ui| {
            ui.label("Kraj pochodzenia");
            ui.add(egui::TextEdit::singleline(&mut self.c_person.country));
            });

        if ui.add(egui::Button::new("Post")).clicked()
        {
            let c_person : person::Person = self.c_person.clone();

            wasm_bindgen_futures::spawn_local(
                async move{
                create_node(c_person).await;
                }
            );

            self.refresh_graph();
        }


            
    });



    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //
    //                                                               QUERIES
    //
    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////


    CollapsingHeader::new("Zapytania").default_open(true).show(ui, |ui| { 

        

    ui.horizontal(|ui| {
        if ui.add(egui::Button::new("Odśwież graf")).clicked()
        {

            if self.task_conn_refresh.is_none() && self.task_node_refresh.is_none()
            {
                self.refresh_graph();
            } 
        }
        });

    ui.horizontal(|ui| {
        ui.label("Przodkowie");
        if ui.add(egui::Button::new("Query")).clicked()
        {
            let id_tgt = self.id_tgt.clone();

            if self.task_query_result.is_none()
            {
            self.task_query_result = Some(Task::spawn( generic_get::< Vec<HashMap<String,String>> >(format!("/person/{}/ancestors",id_tgt.clone()))));
            self.resultbar_s = format!("Przodkowie osoby id={}",id_tgt);
            } 
        }
        });

    ui.horizontal(|ui| {
        ui.label("Najbliższy wspólny przodek");
        if ui.add(egui::Button::new("Query")).clicked()
        {
            let id_tgt = self.id_tgt.clone();
            let id_src = self.id_src.clone();

            if self.task_query_result.is_none()
            {
            self.task_query_result = Some(Task::spawn( generic_get::< Vec<HashMap<String,String>> >(format!("/person/{}/nca/{}",id_src.clone(),id_tgt.clone()))));
            self.resultbar_s = format!("Najbliższy wspólny przodek osób o id={} i id={}",id_src,id_tgt);
            } 
        }
        });

    });


}



fn refresh_nodes(&mut self)
{
    let graph : StableGraph<(),()>  = petgraph::stable_graph::StableGraph::new();
    let mut g : egui_graphs::Graph<(), ()> = egui_graphs::Graph::from(& graph);
    let mut node_map = HashMap::new();

    web_sys::console::log_1(&"Nodes retrieving...".into());

    if let Some(Ok(result)) = self.task_node_refresh.as_mut().unwrap().take_output() {
        web_sys::console::log_1(&"Nodes retrieved, parsing".into());
        if let Ok(nodes) = result
        {
            for p in nodes
            {
            if let Some(id) = p.id {
                let node_id = g.add_node_with_label((),id.to_string());
                node_map.insert(id, node_id);

                web_sys::console::log_1(&id.to_string().into());
            }
            }
            self.g = g;
            self.node_map = node_map;
            self.task_node_refresh = None;
        }
        else {
            web_sys::console::log_1(&"Node parse problem!".into());
            self.task_node_refresh = None;
        }
    } else {
        //web_sys::console::log_1(&"Node retrieval problem!".into());
    }

    
    

}


fn refresh_conns(&mut self)
{

    let mut g : egui_graphs::Graph<(), ()> =self.g.clone();
    web_sys::console::log_1(&"Conns retrieving...".into());

    if let Some(Ok(result)) = self.task_conn_refresh.as_mut().unwrap().take_output() {
        web_sys::console::log_1(&"Conns retrieved, parsing".into());
        if let Ok(links) = result
        {
            for p in links
            {
                g.add_edge_with_label(self.node_map[&p.id_src], self.node_map[&p.id_tgt],(),p.relation.clone());
                web_sys::console::log_1( &pretty_string!("{} -[{}]-> {}",p.id_src,p.relation,p.id_tgt).into() );
            
            }
            self.g = g;
            self.task_conn_refresh = None;
            self.refresh_layout();
        }
        else {
            web_sys::console::log_1(&"Conns parse problem!".into());
            self.task_conn_refresh = None;
        }
    } else {
        //web_sys::console::log_1(&"Node retrieval problem!".into());
    }
}


fn refresh_layout(&mut self)
{
    let mut visited = HashSet::new();
    let mut max_col = 0;
    self.g.g.externals(petgraph::Incoming)
        .collect::<Vec<petgraph::stable_graph::NodeIndex>>()
        .iter()
        .enumerate()
        .for_each(|(i, root_idx)| {
            visited.insert(*root_idx);

            let curr_max_col = build_tree(&mut self.g, &mut visited, root_idx, 0, i);
            if curr_max_col > max_col {
                max_col = curr_max_col;
            };
        });
}

fn refresh_gperson(&mut self)
{
    web_sys::console::log_1(&"Person data retrieving...".into());

    if let Some(Ok(result)) = self.task_gperson_refresh.as_mut().unwrap().take_output() {
        web_sys::console::log_1(&"Person data retrieved, parsing".into());
        if let Ok(person) = result
        {

            self.g_person = person;
            self.task_gperson_refresh = None;
        }
        else {
            web_sys::console::log_1(&"Person data parse problem!".into());
            web_sys::console::log_1(&format!("{:?}", result).into());
            self.task_gperson_refresh = None;
        }
    } 
}

fn refresh_query(&mut self)
{
    web_sys::console::log_1(&"Query data retrieving...".into());

    if let Some(Ok(result)) = self.task_query_result.as_mut().unwrap().take_output() {
        web_sys::console::log_1(&"Query data retrieved, parsing".into());
        if let Ok(table) = result
        {

            self.query_result = table;
            self.task_query_result = None;
        }
        else {
            web_sys::console::log_1(&"Query data parse problem!".into());
            web_sys::console::log_1(&format!("{:?}", result).into());
            show_toast(&format!("Porażka: {:?}.", result));
            self.task_query_result = None;
            self.query_result.clear();
        }
    } 
}


fn process_tasks(&mut self)
{
    if self.task_node_refresh.is_some()
    {
        self.refresh_nodes();
    }
    else if self.task_conn_refresh.is_some()
    {
        self.refresh_conns();
    }

    if self.task_gperson_refresh.is_some()
    {
        self.refresh_gperson();
    }

    if self.task_query_result.is_some()
    {
        self.refresh_query();
    }

}

fn refresh_graph(&mut self)
{
    self.task_node_refresh = Some(Task::spawn( generic_get::<Vec<person::Person>>("/people".to_owned())));
    self.task_conn_refresh = Some(Task::spawn(generic_get::<Vec<person::Link>>("/relations".to_owned())));
}


fn process_inputs(&mut self, ctx: &egui::Context)
{

    if self.id_src.chars().count() > 0 && self.id_tgt.chars().count() > 0 {
        if ctx.input(|i| i.modifiers.alt) 
    {
        

    if ctx.input(|i| i.key_pressed(egui::Key::R))
    {
        self.refresh_graph()
    }
        
    if ctx.input(|i| i.key_pressed(egui::Key::J)) 
    {


        let link_id_src : u32 = self.id_src.parse().unwrap();
        let link_id_tgt : u32 = self.id_tgt.parse().unwrap();

        wasm_bindgen_futures::spawn_local(
            async move{
            create_link(link_id_src.clone(),link_id_tgt.clone()).await;
            }
        );
        
        self.g.add_edge_with_label(self.node_map[&link_id_src], self.node_map[&link_id_tgt],(),"PARENT_OF".to_owned());
    }

    if ctx.input(|i| i.key_pressed(egui::Key::C))
    {


        let link_id_src : u32 = self.id_src.parse().unwrap();
        let link_id_tgt : u32 = self.id_tgt.parse().unwrap();

        wasm_bindgen_futures::spawn_local(
            async move{
            create_link_cousin(link_id_src.clone(),link_id_tgt.clone()).await;
            }
        );
        
        self.g.add_edge_with_label(self.node_map[&link_id_src], self.node_map[&link_id_tgt],(),"COUSIN_OF".to_owned());
    }}

    }

    if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {

        let edge_ids = self.g.selected_edges().to_vec();
                
        self.g.set_selected_edges(Vec::new());
        
        edge_ids.iter().for_each(|edge| {

            if let Some((src,targ)) = self.g.edge_endpoints(*edge)
            {
                web_sys::console::log_1(&format!("deleting {:?} -> {:?}",src,targ).into());
                

                let id_src : u32 = self.node_index_id(src).unwrap();
                let id_tgt : u32 = self.node_index_id(targ).unwrap();

                let mut relation = "child";

                if let Some(graph_edge) =  self.g.edge(*edge)
                {
                    if graph_edge.label() == "COUSIN_OF"
                    {
                        relation = "cousin";
                    }
                }

                wasm_bindgen_futures::spawn_local(
                    async move{
                    generic_delete(format!("/person/{}/{}/{}",id_src,relation,id_tgt)).await;
                    });

            }

            self.g.remove_edge(*edge);
            
        });   
        
        let node_ids = self.g.selected_nodes().to_vec();
        self.g.set_selected_nodes(Vec::new());

        node_ids.iter().for_each(|node|{

            let id_tgt = self.node_index_id(node.clone()).unwrap();
            wasm_bindgen_futures::spawn_local(
                async move{
                generic_delete(format!("/person/{}",id_tgt)).await;
                });

            self.g.remove_node(*node);
        });

        
        
        }

}


fn handle_events(&mut self) {

        
    self.event_consumer.clone().try_iter().for_each(|e| {

        if self.last_events.len() > EVENTS_LIMIT {
            self.last_events.remove(0);
        }
        self.last_events.push(serde_json::to_string(&e).unwrap());
        
        println!("{}",serde_json::to_string(&e).unwrap());
        
        match e
        {
            //egui_graphs::events::Event::NodeDeselect(payload) =>
            //{
            //}
            egui_graphs::events::Event::NodeSelect(payload) =>
            {
                let nodeid : petgraph::prelude::NodeIndex = petgraph::prelude::NodeIndex::new(payload.id);

                self.id_src = self.id_tgt.clone();
                self.id_tgt = self.node_index_id(nodeid).unwrap().to_string();


                self.task_gperson_refresh =  Some(Task::spawn(generic_get::<person::Person>( pretty_string!("/person/{}",self.id_tgt.clone()) )));
            }
            _ => {}
        }
        
        
        
        
        
        
        
        });
}

fn node_index_id(& self,target : petgraph::prelude::NodeIndex) -> Option<u32>
    {
        if let Some( node ) = self.g.node(target.into())
        {
            if let Ok(obtained) = node.label().parse::<u32>()
            {
                return Some(obtained);
            }
            else 
            {
            return None;
            }
        }
        return None;

    }




}




#[cfg(target_arch = "wasm32")]
pub struct Task<T>(std::rc::Rc<std::cell::Cell<Option<std::thread::Result<T>>>>);

#[cfg(target_arch = "wasm32")]
impl<T: 'static> Task<T> {
    pub fn spawn<F: 'static + std::future::Future<Output = T>>(future: F) -> Self {
        let sender = std::rc::Rc::new(std::cell::Cell::new(None));
        let receiver = sender.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let result = future.await; // Directly await the future
            sender.set(Some(Ok(result))); // Store the result in Ok()
        });
        
        Self(receiver)
    }
    pub fn take_output(&self) -> Option<std::thread::Result<T>> {
        self.0.take()
    }

}


pub fn draw_table(ui: &mut egui::Ui, data: & Vec<HashMap<String, String>>) {
    if data.is_empty() {
        ui.label("No data available.");
        return;
    }
    //ui.label("Query result");

    // Extract all unique keys (column headers) from the data
    let mut column_headers: Vec<String> = data.iter()
        .flat_map(|record| record.keys().cloned())
        .collect();
    column_headers.sort();
    column_headers.dedup();

    if let Some(pos) = column_headers.iter().position(|x| {
        let xc : String = x.clone().to_ascii_lowercase();
        xc == "id"
    }) 
    {
        let element = column_headers.remove(pos); // Remove the element from its current position
        column_headers.insert(0, element);       // Insert it at the first position
    }

    egui_extras::TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        //.column( egui_extras::Column::auto().resizable(true)) // Create one column per header
        .columns(egui_extras::Column::remainder().resizable(true),column_headers.len())
        .header(20.0, |mut header| {
            for column_name in &column_headers {
                header.col(|ui: &mut egui::Ui| {
                    ui.heading(column_name);
                });
            }
        })
        .body(|mut body| {
            for record in data {
                body.row(30.0, |mut row| {
                    for column_name in &column_headers {
                        row.col(|ui| {
                            if let Some(value) = record.get(column_name) {
                                ui.label(value);
                            } else {
                                ui.label("NULL");
                            }
                        });
                    }
                });
            }
        });
}

use wasm_bindgen::JsCast;

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn show_toast(message: &str) {
    // Create a new div element to act as the toast notification
    let document = web_sys::window().unwrap().document().unwrap();
    let toast = document.create_element("div").unwrap();
    
    // Set the content and style of the toast
    toast.set_inner_html(message);
    toast.set_attribute("style", "position: fixed; bottom: 20px; left: 20px; padding: 10px; background-color: rgba(0, 0, 0, 0.65); color: red; border-radius: 5px; z-index: 1000;").unwrap();

    // Append the toast to the body
    let body = document.body().unwrap();
    body.append_child(&toast).unwrap();

    // Optionally, remove the toast after a delay
    let toast_clone = toast.clone();
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        toast_clone.remove();
    }) as Box<dyn Fn()>);

    let _ =web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), 4000 + TryInto::<i32>::try_into(message.len()*30).unwrap() );
    closure.forget(); // `forget` ensures that the closure is not dropped prematurely
}





pub async fn create_node(c_person : person::Person)
{
let client = reqwest::Client::new();

let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"));

let res = 
    client
        .post(SERVER_ADDRESS.to_owned() + "/person") // Replace with your endpoint URL
        //.headers(headers)
        //.fetch_mode_no_cors()
        .json(&c_person) // Serialize and send the person object as JSON
        .send().await;

match res {
    Ok(response) => {
        if response.status().is_success() {
            web_sys::console::log_1(&"Node creation success".into());
        } else {
            web_sys::console::log_1(&pretty_string!("Failed to send request: {}", response.status()).into());
            show_toast(&format!("Porażka: {:?}. Należy odświeżyć graf", response.text().await));
            //println!("Failed to send request: {}", response.status());
        }
    }
    Err(e) => {
        //println!("Error occurred: {}", e);
        web_sys::console::log_1(&pretty_string!("Error occurred: {}", e).into());
    }
}
}


pub async fn create_link(id_src : u32, id_tgt : u32)
{
    let client = reqwest::Client::new();

    let res = 
        client
            .post(SERVER_ADDRESS.to_owned() + &pretty_string!("/person/{}/child/{}",id_src,id_tgt) ) // Replace with your endpoint URL
            .send().await;
    
    match res {
        Ok(response) => {
            if response.status().is_success() { web_sys::console::log_1(&"Link creation success".into()); } 
            else { web_sys::console::log_1(&pretty_string!("Failed to send request: {}", response.status()).into()); 
            show_toast(&format!("Porażka: {:?}. Należy odświeżyć graf", response.text().await));
        }
        }
        Err(e) => {
            web_sys::console::log_1(&pretty_string!("Error occurred: {}", e).into());
        }
    }
}

pub async fn create_link_cousin(id_p1 : u32, id_p2 : u32)
{
    let client = reqwest::Client::new();

    let res = 
        client
            .post(SERVER_ADDRESS.to_owned() + &pretty_string!("/person/{}/cousin/{}",id_p1,id_p2) ) // Replace with your endpoint URL
            .send().await;
    
    match res {
        Ok(response) => {
            if response.status().is_success() { web_sys::console::log_1(&"Cousin link creation success".into()); } 
            else { web_sys::console::log_1(&pretty_string!("Failed to send request: {}", response.status()).into()); 
            show_toast(&format!("Porażka: {:?}. Należy odświeżyć graf", response.text().await));}
        }
        Err(e) => {web_sys::console::log_1(&pretty_string!("Error occurred: {}", e).into());}
    }
}



pub async fn generic_get<T>(path : String) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let client = reqwest::Client::new();
    let res = client
        .get(SERVER_ADDRESS.to_owned() + &path)
        .header("access-control-allow-methods", "GET, OPTIONS")
        .header("access-control-allow-origin", "*")
        .send()
        .await?;
    let body = res.text().await?;
    let result: T = serde_json::from_str(&body)?;
    Ok(result)
}


pub async fn generic_delete(path : String) 
{
    let client = reqwest::Client::new();
    let res = client
        .delete(SERVER_ADDRESS.to_owned() + &path)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() { web_sys::console::log_1(&"Deletion success".into()); } 
            else { web_sys::console::log_1(&pretty_string!("Failed to send request: {}", response.status()).into()); }
        }
        Err(e) => {web_sys::console::log_1(&pretty_string!("Error occurred: {}", e).into());}
    }
}




fn build_tree<N, E, Ty, Ix, Dn, De>(
    g: &mut egui_graphs::Graph<N, E, Ty, Ix, Dn, De>,
    visited: &mut HashSet<petgraph::stable_graph::NodeIndex<Ix>>,
    root_idx: &petgraph::stable_graph::NodeIndex<Ix>,
    start_row: usize,
    start_col: usize,
) -> usize
where
    N: Clone,
    E: Clone,
    Ty: petgraph::EdgeType,
    Ix: petgraph::csr::IndexType,
    Dn: egui_graphs::DisplayNode<N, E, Ty, Ix>,
    De: egui_graphs::DisplayEdge<N, E, Ty, Ix, Dn>,
{
    let y = start_row * 50;
    let x = start_col * 50;

    let node = &mut g.g[*root_idx];
    node.set_location(egui::Pos2::new(x as f32, y as f32));

    let mut max_col = start_col;
    g.g.neighbors_directed(*root_idx, petgraph::Direction::Outgoing)
        .collect::<Vec<petgraph::stable_graph::NodeIndex<Ix>>>()
        .iter()
        .enumerate()
        .for_each(|(i, neighbour_idx)| {
            if visited.contains(neighbour_idx) {
                return;
            };

            visited.insert(*neighbour_idx);

            let curr_max_col = build_tree(g, visited, neighbour_idx, start_row + 1, start_col + i);
            if curr_max_col > max_col {
                max_col = curr_max_col;
            };
        });

    max_col
}