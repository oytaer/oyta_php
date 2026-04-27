//! 分词器模块
//!
//! 提供文本分词功能，支持中英文分词

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 词项类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    /// 普通词
    Word,
    /// 数字
    Number,
    /// 中文词
    Chinese,
    /// 英文词
    English,
    /// 标点符号
    Punctuation,
    /// 空白字符
    Whitespace,
    /// URL
    Url,
    /// 邮箱
    Email,
    /// 日期
    Date,
    /// 自定义
    Custom,
}

/// 词项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// 词项文本
    pub text: String,
    /// 词项类型
    pub token_type: TokenType,
    /// 起始位置
    pub start: usize,
    /// 结束位置
    pub end: usize,
    /// 词项位置索引
    pub position: usize,
}

impl Token {
    /// 创建新词项
    pub fn new(text: String, token_type: TokenType, start: usize, end: usize, position: usize) -> Self {
        Self {
            text,
            token_type,
            start,
            end,
            position,
        }
    }
    
    /// 转换为小写
    pub fn to_lowercase(&self) -> String {
        self.text.to_lowercase()
    }
    
    /// 获取长度
    pub fn len(&self) -> usize {
        self.text.len()
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// 分词器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizerConfig {
    /// 是否转换为小写
    pub lowercase: bool,
    /// 是否移除标点符号
    pub remove_punctuation: bool,
    /// 是否移除数字
    pub remove_numbers: bool,
    /// 是否移除停用词
    pub remove_stopwords: bool,
    /// 最小词长
    pub min_token_length: usize,
    /// 最大词长
    pub max_token_length: usize,
    /// 中文分词模式
    pub chinese_mode: ChineseTokenizeMode,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            lowercase: true,
            remove_punctuation: true,
            remove_numbers: false,
            remove_stopwords: true,
            min_token_length: 1,
            max_token_length: 100,
            chinese_mode: ChineseTokenizeMode::Ngram(2),
        }
    }
}

/// 中文分词模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChineseTokenizeMode {
    /// 单字分词
    Character,
    /// N-gram 分词
    Ngram(usize),
    /// 最大匹配
    MaxMatch,
}

/// 分词器
/// 提供文本分词功能
#[derive(Debug)]
pub struct Tokenizer {
    /// 配置
    config: TokenizerConfig,
    /// 停用词表
    stopwords: HashMap<String, bool>,
}

impl Tokenizer {
    /// 创建新的分词器
    pub fn new() -> Self {
        Self::with_config(TokenizerConfig::default())
    }
    
    /// 使用配置创建分词器
    pub fn with_config(config: TokenizerConfig) -> Self {
        let mut tokenizer = Self {
            config,
            stopwords: HashMap::new(),
        };
        
        // 加载默认停用词
        tokenizer.load_default_stopwords();
        
        tokenizer
    }
    
    /// 加载默认停用词
    fn load_default_stopwords(&mut self) {
        // 英文停用词
        let english_stopwords = [
            "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "is", "are", "was", "were", "be", "been",
            "being", "have", "has", "had", "do", "does", "did", "will", "would",
            "could", "should", "may", "might", "must", "shall", "can", "this",
            "that", "these", "those", "it", "its", "they", "them", "their",
            "he", "him", "his", "she", "her", "i", "me", "my", "we", "us", "our",
            "you", "your", "as", "if", "then", "else", "when", "where", "what",
            "which", "who", "whom", "how", "why", "not", "no", "yes", "all",
            "each", "every", "both", "few", "more", "most", "other", "some",
            "such", "only", "own", "same", "so", "than", "too", "very", "just",
        ];
        
        for word in english_stopwords {
            self.stopwords.insert(word.to_string(), true);
        }
        
        // 中文停用词
        let chinese_stopwords = [
            "的", "了", "和", "是", "在", "有", "我", "他", "她", "它",
            "这", "那", "之", "与", "以", "及", "或", "但", "而", "也",
            "就", "都", "又", "要", "能", "会", "可", "上", "下", "中",
            "来", "去", "说", "对", "很", "被", "把", "从", "到", "向",
            "着", "过", "个", "们", "等", "还", "让", "给", "比", "很",
        ];
        
        for word in chinese_stopwords {
            self.stopwords.insert(word.to_string(), true);
        }
    }
    
    /// 添加停用词
    pub fn add_stopword(&mut self, word: &str) {
        self.stopwords.insert(word.to_string(), true);
    }
    
    /// 移除停用词
    pub fn remove_stopword(&mut self, word: &str) {
        self.stopwords.remove(word);
    }
    
    /// 检查是否为停用词
    fn is_stopword(&self, word: &str) -> bool {
        self.config.remove_stopwords && self.stopwords.contains_key(word)
    }
    
    /// 分词
    /// 
    /// # 参数
    /// - `text`: 要分词的文本
    pub fn tokenize(&self, text: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut position = 0;
        
        // 遍历文本
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let c = chars[i];
            
            // 检查字符类型
            if c.is_whitespace() {
                // 跳过空白字符
                if !self.config.remove_punctuation {
                    tokens.push(Token::new(
                        c.to_string(),
                        TokenType::Whitespace,
                        i,
                        i + 1,
                        position,
                    ));
                    position += 1;
                }
                i += 1;
            } else if c.is_ascii_punctuation() || Self::is_chinese_punctuation(c) {
                // 标点符号
                if !self.config.remove_punctuation {
                    tokens.push(Token::new(
                        c.to_string(),
                        TokenType::Punctuation,
                        i,
                        i + 1,
                        position,
                    ));
                    position += 1;
                }
                i += 1;
            } else if c.is_ascii_digit() {
                // 数字
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                
                let text: String = chars[start..i].iter().collect();
                
                if !self.config.remove_numbers {
                    tokens.push(Token::new(
                        text,
                        TokenType::Number,
                        start,
                        i,
                        position,
                    ));
                    position += 1;
                }
            } else if c.is_ascii_alphabetic() {
                // 英文单词
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i].is_ascii_digit()) {
                    i += 1;
                }
                
                let text: String = chars[start..i].iter().collect();
                let token_text = if self.config.lowercase {
                    text.to_lowercase()
                } else {
                    text
                };
                
                // 检查长度和停用词
                if token_text.len() >= self.config.min_token_length
                    && token_text.len() <= self.config.max_token_length
                    && !self.is_stopword(&token_text)
                {
                    tokens.push(Token::new(
                        token_text,
                        TokenType::English,
                        start,
                        i,
                        position,
                    ));
                    position += 1;
                }
            } else if Self::is_chinese_char(c) {
                // 中文字符
                match self.config.chinese_mode {
                    ChineseTokenizeMode::Character => {
                        // 单字分词
                        let text = c.to_string();
                        if !self.is_stopword(&text) {
                            tokens.push(Token::new(
                                text,
                                TokenType::Chinese,
                                i,
                                i + 1,
                                position,
                            ));
                            position += 1;
                        }
                        i += 1;
                    }
                    ChineseTokenizeMode::Ngram(n) => {
                        // N-gram 分词
                        let start = i;
                        let mut end = i;
                        while end < chars.len() && Self::is_chinese_char(chars[end]) && end - start < n {
                            end += 1;
                        }
                        
                        let text: String = chars[start..end].iter().collect();
                        if text.len() >= self.config.min_token_length && !self.is_stopword(&text) {
                            tokens.push(Token::new(
                                text,
                                TokenType::Chinese,
                                start,
                                end,
                                position,
                            ));
                            position += 1;
                        }
                        i += 1;
                    }
                    ChineseTokenizeMode::MaxMatch => {
                        // 最大匹配（简化实现）
                        let start = i;
                        while i < chars.len() && Self::is_chinese_char(chars[i]) {
                            i += 1;
                        }
                        
                        let text: String = chars[start..i].iter().collect();
                        if !self.is_stopword(&text) {
                            tokens.push(Token::new(
                                text,
                                TokenType::Chinese,
                                start,
                                i,
                                position,
                            ));
                            position += 1;
                        }
                    }
                }
            } else {
                // 其他字符
                i += 1;
            }
        }
        
        tokens
    }
    
    /// 检查是否为中文字符
    fn is_chinese_char(c: char) -> bool {
        matches!(c,
            '\u{4E00}'..='\u{9FFF}' |      // CJK 统一汉字
            '\u{3400}'..='\u{4DBF}' |      // CJK 扩展 A
            '\u{20000}'..='\u{2A6DF}' |    // CJK 扩展 B
            '\u{2A700}'..='\u{2B73F}' |    // CJK 扩展 C
            '\u{2B740}'..='\u{2B81F}' |    // CJK 扩展 D
            '\u{2B820}'..='\u{2CEAF}' |    // CJK 扩展 E
            '\u{F900}'..='\u{FAFF}' |      // CJK 兼容汉字
            '\u{2F800}'..='\u{2FA1F}'      // CJK 兼容补充
        )
    }
    
    /// 检查是否为中文标点
    fn is_chinese_punctuation(c: char) -> bool {
        matches!(c,
            '，' | '。' | '、' | '；' | '：' | '？' | '！' |
            '"' | '"' | '\'' | '\'' | '（' | '）' | '【' | '】' |
            '《' | '》' | '〈' | '〉' | '「' | '」' | '『' | '』' |
            '﹃' | '﹄' | '〔' | '〕' | '…' | '—' | '～' | '·'
        )
    }
    
    /// 分词并返回文本列表
    /// 
    /// # 参数
    /// - `text`: 要分词的文本
    pub fn tokenize_to_strings(&self, text: &str) -> Vec<String> {
        self.tokenize(text)
            .into_iter()
            .map(|t| t.text)
            .collect()
    }
    
    /// 获取配置
    pub fn config(&self) -> &TokenizerConfig {
        &self.config
    }
    
    /// 更新配置
    pub fn set_config(&mut self, config: TokenizerConfig) {
        self.config = config;
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试英文分词
    #[test]
    fn test_english_tokenize() {
        let tokenizer = Tokenizer::new();
        
        let tokens = tokenizer.tokenize("Hello World!");
        
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, "world");
    }
    
    /// 测试中文分词
    #[test]
    fn test_chinese_tokenize() {
        let tokenizer = Tokenizer::new();
        
        let tokens = tokenizer.tokenize("你好世界");
        
        // 使用 n-gram 分词
        assert!(!tokens.is_empty());
    }
    
    /// 测试停用词过滤
    #[test]
    fn test_stopwords() {
        let tokenizer = Tokenizer::new();
        
        let tokens = tokenizer.tokenize("This is a test");
        
        // "this", "is", "a" 是停用词
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "test");
    }
    
    /// 测试数字分词
    #[test]
    fn test_number_tokenize() {
        let mut config = TokenizerConfig::default();
        config.remove_numbers = false;
        
        let tokenizer = Tokenizer::with_config(config);
        
        let tokens = tokenizer.tokenize("Price: 100.50 dollars");
        
        assert!(tokens.iter().any(|t| t.text == "100.50"));
    }
    
    /// 测试混合文本
    #[test]
    fn test_mixed_tokenize() {
        let tokenizer = Tokenizer::new();
        
        let tokens = tokenizer.tokenize("Hello 你好 World 世界");
        
        assert!(!tokens.is_empty());
    }
}
