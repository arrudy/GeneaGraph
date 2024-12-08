#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Person{
    pub id : Option<u32>,
    pub name : String,
    pub last_name: String,
    pub birth_dt: chrono::naive::NaiveDate,
    pub death_dt: Option<chrono::naive::NaiveDate>,
    pub country: String
}



#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Link{
    pub id_src : u32,
    pub relation : String,
    pub id_tgt : u32
}