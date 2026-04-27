//! 查询解析模块
//!
//! 解析搜索查询并生成查询计划

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::index::DocumentId;

/// 查询类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Query {
    /// 词项查询
    Term {
        /// 词项
        term: String,
    },
    /// 短语查询
    Phrase {
        /// 短语
        phrase: String,
        /// 词项列表
        terms: Vec<String>,
    },
    /// 布尔查询
    Boolean {
        /// 必须包含的词项
        must: Vec<Box<Query>>,
        /// 应该包含的词项
        should: Vec<Box<Query>>,
        /// 不能包含的词项
        must_not: Vec<Box<Query>>,
    },
    /// 通配符查询
    Wildcard {
        /// 模式
        pattern: String,
    },
    /// 模糊查询
    Fuzzy {
        /// 词项
        term: String,
        /// 最大编辑距离
        max_distance: usize,
    },
    /// 范围查询
    Range {
        /// 字段名
        field: String,
        /// 起始值
        from: Option<String>,
        /// 结束值
        to: Option<String>,
        /// 是否包含起始
        include_from: bool,
        /// 是否包含结束
        include_to: bool,
    },
    /// 前缀查询
    Prefix {
        /// 前缀
        prefix: String,
    },
    /// 正则查询
    Regex {
        /// 正则表达式
        pattern: String,
    },
    /// 匹配所有
    MatchAll,
    /// 匹配无
    MatchNone,
}

impl Query {
    /// 创建词项查询
    pub fn term(term: &str) -> Self {
        Self::Term {
            term: term.to_string(),
        }
    }
    
    /// 创建短语查询
    pub fn phrase(phrase: &str) -> Self {
        Self::Phrase {
            phrase: phrase.to_string(),
            terms: phrase.split_whitespace().map(|s| s.to_string()).collect(),
        }
    }
    
    /// 创建布尔查询
    pub fn boolean() -> BooleanQueryBuilder {
        BooleanQueryBuilder::new()
    }
    
    /// 创建通配符查询
    pub fn wildcard(pattern: &str) -> Self {
        Self::Wildcard {
            pattern: pattern.to_string(),
        }
    }
    
    /// 创建模糊查询
    pub fn fuzzy(term: &str, max_distance: usize) -> Self {
        Self::Fuzzy {
            term: term.to_string(),
            max_distance,
        }
    }
    
    /// 创建前缀查询
    pub fn prefix(prefix: &str) -> Self {
        Self::Prefix {
            prefix: prefix.to_string(),
        }
    }
    
    /// 创建匹配所有查询
    pub fn match_all() -> Self {
        Self::MatchAll
    }
    
    /// 创建匹配无查询
    pub fn match_none() -> Self {
        Self::MatchNone
    }
}

/// 布尔查询构建器
pub struct BooleanQueryBuilder {
    must: Vec<Box<Query>>,
    should: Vec<Box<Query>>,
    must_not: Vec<Box<Query>>,
}

impl BooleanQueryBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            must: Vec::new(),
            should: Vec::new(),
            must_not: Vec::new(),
        }
    }
    
    /// 添加必须匹配的查询
    pub fn must(mut self, query: Query) -> Self {
        self.must.push(Box::new(query));
        self
    }
    
    /// 添加应该匹配的查询
    pub fn should(mut self, query: Query) -> Self {
        self.should.push(Box::new(query));
        self
    }
    
    /// 添加不能匹配的查询
    pub fn must_not(mut self, query: Query) -> Self {
        self.must_not.push(Box::new(query));
        self
    }
    
    /// 构建布尔查询
    pub fn build(self) -> Query {
        Query::Boolean {
            must: self.must,
            should: self.should,
            must_not: self.must_not,
        }
    }
}

impl Default for BooleanQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// 匹配的文档 ID 列表
    pub doc_ids: Vec<DocumentId>,
    /// 总匹配数
    pub total: usize,
    /// 查询耗时（微秒）
    pub took_us: u64,
    /// 查询信息
    pub info: HashMap<String, String>,
}

impl QueryResult {
    /// 创建新的查询结果
    pub fn new(doc_ids: Vec<DocumentId>) -> Self {
        let total = doc_ids.len();
        Self {
            doc_ids,
            total,
            took_us: 0,
            info: HashMap::new(),
        }
    }
    
    /// 设置查询耗时
    pub fn with_took(mut self, took_us: u64) -> Self {
        self.took_us = took_us;
        self
    }
    
    /// 添加信息
    pub fn add_info(mut self, key: &str, value: &str) -> Self {
        self.info.insert(key.to_string(), value.to_string());
        self
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.doc_ids.is_empty()
    }
    
    /// 获取文档数量
    pub fn len(&self) -> usize {
        self.doc_ids.len()
    }
}

/// 查询解析器
/// 解析查询字符串并生成查询对象
#[derive(Debug)]
pub struct QueryParser {
    /// 默认字段
    default_field: String,
    /// 默认操作符
    default_operator: QueryOperator,
}

/// 查询操作符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryOperator {
    /// AND 操作
    And,
    /// OR 操作
    Or,
}

impl QueryParser {
    /// 创建新的查询解析器
    pub fn new() -> Self {
        Self {
            default_field: "content".to_string(),
            default_operator: QueryOperator::Or,
        }
    }
    
    /// 设置默认字段
    pub fn default_field(mut self, field: &str) -> Self {
        self.default_field = field.to_string();
        self
    }
    
    /// 设置默认操作符
    pub fn default_operator(mut self, operator: QueryOperator) -> Self {
        self.default_operator = operator;
        self
    }
    
    /// 解析查询字符串
    /// 
    /// # 参数
    /// - `query_str`: 查询字符串
    pub fn parse(&self, query_str: &str) -> Result<Query, QueryParseError> {
        let query_str = query_str.trim();
        
        // 空查询
        if query_str.is_empty() {
            return Ok(Query::match_all());
        }
        
        // 检查特殊语法
        if query_str.starts_with("NOT ") {
            // NOT 查询
            let rest = &query_str[4..];
            let inner = self.parse(rest)?;
            return Ok(Query::boolean().must_not(inner).build());
        }
        
        // 检查引号短语
        if query_str.starts_with('"') && query_str.ends_with('"') && query_str.len() > 2 {
            let phrase = &query_str[1..query_str.len() - 1];
            return Ok(Query::phrase(phrase));
        }
        
        // 检查通配符
        if query_str.contains('*') || query_str.contains('?') {
            return Ok(Query::wildcard(query_str));
        }
        
        // 检查模糊查询
        if query_str.ends_with('~') {
            let term = &query_str[..query_str.len() - 1];
            return Ok(Query::fuzzy(term, 2));
        }
        
        // 检查前缀
        if query_str.ends_with('*') {
            let prefix = &query_str[..query_str.len() - 1];
            return Ok(Query::prefix(prefix));
        }
        
        // 检查布尔操作符
        if query_str.contains(" AND ") || query_str.contains(" OR ") || query_str.contains(" NOT ") {
            return self.parse_boolean(query_str);
        }
        
        // 检查多个词项
        let terms: Vec<&str> = query_str.split_whitespace().collect();
        
        if terms.len() == 1 {
            // 单个词项
            Ok(Query::term(terms[0]))
        } else {
            // 多个词项
            match self.default_operator {
                QueryOperator::And => {
                    let mut builder = BooleanQueryBuilder::new();
                    for term in terms {
                        builder = builder.must(Query::term(term));
                    }
                    Ok(builder.build())
                }
                QueryOperator::Or => {
                    let mut builder = BooleanQueryBuilder::new();
                    for term in terms {
                        builder = builder.should(Query::term(term));
                    }
                    Ok(builder.build())
                }
            }
        }
    }
    
    /// 解析布尔查询
    fn parse_boolean(&self, query_str: &str) -> Result<Query, QueryParseError> {
        let mut builder = BooleanQueryBuilder::new();
        
        // 简单解析（实际应该使用更复杂的解析器）
        let parts: Vec<&str> = query_str.split(" AND ").collect();
        
        for part in parts {
            let part = part.trim();
            
            if part.contains(" OR ") {
                // OR 子句
                let or_terms: Vec<&str> = part.split(" OR ").collect();
                let mut or_builder = BooleanQueryBuilder::new();
                for term in or_terms {
                    or_builder = or_builder.should(Query::term(term.trim()));
                }
                builder = builder.must(or_builder.build());
            } else if part.starts_with("NOT ") {
                // NOT 子句
                let term = &part[4..];
                builder = builder.must_not(Query::term(term.trim()));
            } else {
                // 普通词项
                builder = builder.must(Query::term(part));
            }
        }
        
        Ok(builder.build())
    }
}

impl Default for QueryParser {
    fn default() -> Self {
        Self::new()
    }
}

/// 查询解析错误
#[derive(Debug, Clone)]
pub enum QueryParseError {
    /// 语法错误
    SyntaxError(String),
    /// 不支持的查询类型
    UnsupportedQuery(String),
    /// 无效的字段名
    InvalidField(String),
    /// 无效的值
    InvalidValue(String),
}

impl std::fmt::Display for QueryParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SyntaxError(msg) => write!(f, "查询语法错误: {}", msg),
            Self::UnsupportedQuery(msg) => write!(f, "不支持的查询类型: {}", msg),
            Self::InvalidField(field) => write!(f, "无效的字段名: {}", field),
            Self::InvalidValue(value) => write!(f, "无效的值: {}", value),
        }
    }
}

impl std::error::Error for QueryParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试词项查询
    #[test]
    fn test_term_query() {
        let parser = QueryParser::new();
        
        let query = parser.parse("hello").unwrap();
        
        match query {
            Query::Term { term } => assert_eq!(term, "hello"),
            _ => panic!("Expected term query"),
        }
    }
    
    /// 测试短语查询
    #[test]
    fn test_phrase_query() {
        let parser = QueryParser::new();
        
        let query = parser.parse("\"hello world\"").unwrap();
        
        match query {
            Query::Phrase { phrase, .. } => assert_eq!(phrase, "hello world"),
            _ => panic!("Expected phrase query"),
        }
    }
    
    /// 测试布尔查询
    #[test]
    fn test_boolean_query() {
        let parser = QueryParser::new();
        
        let query = parser.parse("hello AND world").unwrap();
        
        match query {
            Query::Boolean { must, .. } => assert_eq!(must.len(), 2),
            _ => panic!("Expected boolean query"),
        }
    }
    
    /// 测试通配符查询
    #[test]
    fn test_wildcard_query() {
        let parser = QueryParser::new();
        
        let query = parser.parse("hel*").unwrap();
        
        match query {
            Query::Wildcard { pattern } => assert_eq!(pattern, "hel*"),
            _ => panic!("Expected wildcard query"),
        }
    }
    
    /// 测试模糊查询
    #[test]
    fn test_fuzzy_query() {
        let parser = QueryParser::new();
        
        let query = parser.parse("hello~").unwrap();
        
        match query {
            Query::Fuzzy { term, max_distance } => {
                assert_eq!(term, "hello");
                assert_eq!(max_distance, 2);
            }
            _ => panic!("Expected fuzzy query"),
        }
    }
    
    /// 测试查询结果
    #[test]
    fn test_query_result() {
        let result = QueryResult::new(vec![DocumentId(1), DocumentId(2)])
            .with_took(100)
            .add_info("test", "value");
        
        assert_eq!(result.len(), 2);
        assert_eq!(result.took_us, 100);
        assert_eq!(result.info.get("test"), Some(&"value".to_string()));
    }
}
