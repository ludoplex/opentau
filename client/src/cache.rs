use redis::Commands;

pub struct Cache {
    stop_at: usize, // TODO: document why we need this
    redis: redis::Connection,
}

impl Cache {
    pub fn new(redis_url: &str, stop_at: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_connection()?;
        Ok(Self {
            redis: conn,
            stop_at,
        })
    }

    /// Stores the given query-result pair in the cache.
    /// auery is a (input: &str, num_comps: usize, retries: usize) tuple.
    /// result is a Vec<String> of the type-checked completions
    pub fn store(
        &mut self,
        query: (&str, usize, usize),
        result: &Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let key = self.to_key(query);
        let value = serde_json::json!(result).to_string();

        self.redis.set(key, value)?;
        Ok(())
    }

    /// Returns the cached result for the given query, if it exists.
    pub fn retrieve(
        &mut self,
        query: (&str, usize, usize),
    ) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
        let key = self.to_key(query);

        let result: Option<String> = self.redis.get(key)?;
        Ok(result.map(|s| serde_json::from_str(&s).unwrap()))
    }

    fn to_key(&self, query: (&str, usize, usize)) -> String {
        serde_json::json!({
            "query": query.0,
            "num_comps": query.1,
            "retries": query.2,
            "stop_at": self.stop_at,
        })
        .to_string()
    }
}
