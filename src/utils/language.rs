use crate::bot::core::structs::Data;
use crate::db::users::get_language_code;
use logfather::error;


pub async fn get_language(data: &Data, user_id: &String) -> String {
    let mut language_cache = data.language_cache.lock().await;
    if let Some(language) = language_cache.get(user_id) {
        return language.clone();
    }

    match get_language_code(&data.db_pool, user_id).await {
        Ok(language_code) => {
            language_cache.put(user_id.clone(), language_code.clone());
            language_code.clone()
        },
        Err(e) => {
            error!("Failed to get language code: {:?}", e);
            String::from("en")
        }
        
    }
}

pub async fn set_language(data: &Data, user_id: &String, language_code: &str) {
    let mut language_cache = data.language_cache.lock().await;
    language_cache.put(user_id.clone(), language_code.to_string());
}