use crate::app::llm::{Message, MessageType, LLM};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use crate::helper::init::warn;
use std::fs;
use select::document::Document;
use select::predicate::{Name, Class};
use regex::Regex;

pub async fn ask_wiki(messages: &Vec<Message>) -> Result<String, Box<dyn std::error::Error>> {
    let wiki_search = LLM::new("config/wiki/wiki-search.json");
    let wiki_best = LLM::new("config/wiki/wiki-best.json");
    let wiki_resume = LLM::new("config/wiki/wiki-resume.json");

    let settings: serde_json::Value = serde_json::from_str(&fs::read_to_string("config/wiki/wiki.json").unwrap()).unwrap();
    let wiki_url: String = settings.get("wiki_url").unwrap().to_string().replace("\"", "");
    let zim_name: String = settings.get("zim_name").unwrap().to_string().replace("\"", "");

    // Search articles corresponding to user query
    let user_query: Message = messages.last().unwrap().clone();
    let articles: Vec<String> = search_articles(user_query.clone(), wiki_search, &wiki_url, &zim_name).await?;

    // Find best article to respond user query
    let best_article_content = find_get_best_article(articles, &user_query.content, wiki_best, &wiki_url, &zim_name).await?;

    // Resume article and create the response
    let messages = vec![
        Message::new(MessageType::SYSTEM, wiki_resume.system_prompt.clone()),
        Message::new(MessageType::USER, format!("The users query is: {}", user_query.content)),
        Message::new(MessageType::USER, format!("The search results are: {}", best_article_content)),
    ];
    let query_response: String = wiki_resume.ask(&messages).await.unwrap();

    Ok(query_response)
}

async fn search_articles(user_query: Message, search_llm: LLM, wiki_url: &String, zim_name: &String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Use LLM to create 4 queries and fetch articles with those 4 queries
    let messages = vec![
        Message::new(MessageType::SYSTEM, search_llm.system_prompt.clone()),
        user_query,
    ];
    let result = search_llm.ask_tools(&messages).await?;

    let queries: Vec<String> = result[0]["function"]["arguments"]["queries"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();

    // Search articles on wikipedia API
    let mut articles: Vec<String> = Vec::new();
    for query in queries.iter() {
        warn(query.clone());

        // Request kiwix API for articles matching query
        let encoded_query = utf8_percent_encode(&query, NON_ALPHANUMERIC).to_string(); 
        let client = reqwest::Client::new();
        let url = format!("{}/search?books.name={}&pattern={}", wiki_url, zim_name, encoded_query);
        let body = client.get(url).send().await?.text().await?;

        // Select every article corresponding to the query
        let document = Document::from(body.as_str());

        // Select articles title from the query
        let results_div = document.find(Class("results")).next().unwrap();
        for node in results_div.find(Name("a")) {
            let article = node.text();
            articles.push(article.clone());
        }
    }
    Ok(articles)
}

async fn find_get_best_article(articles: Vec<String>, user_query: &String, best_llm: LLM, wiki_url: &String, zim_name: &String) -> Result<String, Box<dyn std::error::Error>> {
    // Create a string with all the articles title
    let mut articles_headings: String = String::new();
    for article in articles {
        articles_headings = format!("{}, {}", &articles_headings, article);
    }

    let messages = vec![
        Message::new(MessageType::SYSTEM, best_llm.system_prompt.clone()),
        Message::new(MessageType::USER, format!("The user's query is: {}. Here are the headings:\n{}\n\nPlease select the most relevant heading. Output the heading only and nothing else.", user_query, articles_headings))];
    let best_article = best_llm.ask(&messages).await?;

    // wiki query get article content & parse
    let client = reqwest::Client::new();
    let url: String = format!("{}/content/{}/A/{}", wiki_url, zim_name, best_article.replace("*","").replace(" ", "_"));
    let body = client.get(url).send().await?.text().await?;
    let content = extract_text_from_tags(&body);

    Ok(content)
}

fn extract_text_from_tags(html: &str) -> String {
    // Créer une expression régulière pour trouver le contenu dans les balises <p>, <h1>, <h2>, <h3>
    let re = Regex::new(r#"<p[^>]*>(.*?)</p>|<h1[^>]*>(.*?)</h1>|<h2[^>]*>(.*?)</h2>|<h3[^>]*>(.*?)</h3>"#).unwrap();

    // Utiliser l'expression régulière pour capturer le contenu des balises <p>, <h1>, <h2>, <h3>
    let text = re.captures_iter(html)
        .flat_map(|cap| {
            // Trouver le premier groupe capturé non vide (parmi cap[1] à cap[4])
            (1..=4)
                .filter_map(|i| cap.get(i))
                .map(|m| m.as_str())
                .flat_map(|s| s.split_whitespace())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>() // collect words
        .join(" "); // join with spaces
    text
}
