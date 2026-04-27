//! 排序器模块
//!
//! 实现搜索结果的排序算法

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::index::{DocumentId, Document, InvertedIndex};

/// 评分方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoringMethod {
    /// BM25 算法
    BM25,
    /// TF-IDF 算法
    TfIdf,
    /// 纯词项频率
    TermFrequency,
    /// 文档长度归一化
    DocLengthNorm,
    /// 自定义评分
    Custom,
}

/// 评分文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredDocument {
    /// 文档 ID
    pub doc_id: DocumentId,
    /// 评分
    pub score: f64,
    /// 文档（可选）
    pub document: Option<Document>,
}

impl ScoredDocument {
    /// 创建新的评分文档
    pub fn new(doc_id: DocumentId, score: f64) -> Self {
        Self {
            doc_id,
            score,
            document: None,
        }
    }
    
    /// 设置文档
    pub fn with_document(mut self, document: Document) -> Self {
        self.document = Some(document);
        self
    }
}

impl PartialEq for ScoredDocument {
    fn eq(&self, other: &Self) -> bool {
        self.doc_id == other.doc_id
    }
}

impl Eq for ScoredDocument {}

impl PartialOrd for ScoredDocument {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredDocument {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 按评分降序排序
        other.score.partial_cmp(&self.score).unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// 排序器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankerConfig {
    /// 评分方法
    pub method: ScoringMethod,
    /// BM25 参数 k1
    pub bm25_k1: f64,
    /// BM25 参数 b
    pub bm25_b: f64,
    /// 是否返回文档内容
    pub return_documents: bool,
    /// 最大返回数量
    pub max_results: usize,
}

impl Default for RankerConfig {
    fn default() -> Self {
        Self {
            method: ScoringMethod::BM25,
            bm25_k1: 1.2,
            bm25_b: 0.75,
            return_documents: true,
            max_results: 100,
        }
    }
}

/// 排序器
/// 对搜索结果进行评分和排序
#[derive(Debug)]
pub struct Ranker {
    /// 配置
    config: RankerConfig,
}

impl Ranker {
    /// 创建新的排序器
    pub fn new() -> Self {
        Self::with_config(RankerConfig::default())
    }
    
    /// 使用配置创建排序器
    pub fn with_config(config: RankerConfig) -> Self {
        Self { config }
    }
    
    /// 对搜索结果进行评分和排序
    /// 
    /// # 参数
    /// - `index`: 倒排索引
    /// - `doc_ids`: 文档 ID 列表
    /// - `query_terms`: 查询词项列表
    pub fn rank(
        &self,
        index: &InvertedIndex,
        doc_ids: Vec<DocumentId>,
        query_terms: &[String],
    ) -> Vec<ScoredDocument> {
        // 计算每个文档的评分
        let mut scored_docs: Vec<ScoredDocument> = doc_ids
            .into_iter()
            .map(|doc_id| {
                let score = self.calculate_score(index, doc_id, query_terms);
                let mut scored = ScoredDocument::new(doc_id, score);
                
                if self.config.return_documents {
                    if let Some(doc) = index.get_document(doc_id) {
                        scored.document = Some(doc);
                    }
                }
                
                scored
            })
            .collect();
        
        // 按评分排序
        scored_docs.sort();
        
        // 限制返回数量
        scored_docs.truncate(self.config.max_results);
        
        scored_docs
    }
    
    /// 计算文档评分
    fn calculate_score(
        &self,
        index: &InvertedIndex,
        doc_id: DocumentId,
        query_terms: &[String],
    ) -> f64 {
        match self.config.method {
            ScoringMethod::BM25 => self.calculate_bm25(index, doc_id, query_terms),
            ScoringMethod::TfIdf => self.calculate_tfidf(index, doc_id, query_terms),
            ScoringMethod::TermFrequency => self.calculate_tf(index, doc_id, query_terms),
            ScoringMethod::DocLengthNorm => self.calculate_doc_length_norm(index, doc_id, query_terms),
            ScoringMethod::Custom => self.calculate_custom(index, doc_id, query_terms),
        }
    }
    
    /// 计算 BM25 评分
    fn calculate_bm25(
        &self,
        index: &InvertedIndex,
        doc_id: DocumentId,
        query_terms: &[String],
    ) -> f64 {
        // 获取文档总数
        let total_docs = index.document_count() as f64;
        if total_docs == 0.0 {
            return 0.0;
        }
        
        // 计算平均文档长度
        let avg_doc_length = self.calculate_avg_doc_length(index);
        
        // 获取文档长度
        let doc_length = self.get_doc_length(index, doc_id);
        
        // BM25 评分
        let mut score = 0.0;
        
        for term in query_terms {
            // 获取词项的文档频率
            let df = index.doc_frequency(term) as f64;
            if df == 0.0 {
                continue;
            }
            
            // 计算 IDF
            let idf = ((total_docs - df + 0.5) / (df + 0.5) + 1.0).ln();
            
            // 获取词项频率
            let tf = self.get_term_frequency(index, term, doc_id);
            if tf == 0.0 {
                continue;
            }
            
            // BM25 公式
            let numerator = tf * (self.config.bm25_k1 + 1.0);
            let denominator = tf + self.config.bm25_k1 * (
                1.0 - self.config.bm25_b + self.config.bm25_b * (doc_length / avg_doc_length)
            );
            
            score += idf * numerator / denominator;
        }
        
        score
    }
    
    /// 计算 TF-IDF 评分
    fn calculate_tfidf(
        &self,
        index: &InvertedIndex,
        doc_id: DocumentId,
        query_terms: &[String],
    ) -> f64 {
        // 获取文档总数
        let total_docs = index.document_count() as f64;
        if total_docs == 0.0 {
            return 0.0;
        }
        
        let mut score = 0.0;
        
        for term in query_terms {
            // 获取词项的文档频率
            let df = index.doc_frequency(term) as f64;
            if df == 0.0 {
                continue;
            }
            
            // 计算 IDF
            let idf = (total_docs / df).ln();
            
            // 获取词项频率
            let tf = self.get_term_frequency(index, term, doc_id);
            
            score += tf * idf;
        }
        
        score
    }
    
    /// 计算纯词项频率评分
    fn calculate_tf(
        &self,
        index: &InvertedIndex,
        doc_id: DocumentId,
        query_terms: &[String],
    ) -> f64 {
        let mut score = 0.0;
        
        for term in query_terms {
            score += self.get_term_frequency(index, term, doc_id);
        }
        
        score
    }
    
    /// 计算文档长度归一化评分
    fn calculate_doc_length_norm(
        &self,
        index: &InvertedIndex,
        doc_id: DocumentId,
        query_terms: &[String],
    ) -> f64 {
        let doc_length = self.get_doc_length(index, doc_id);
        if doc_length == 0.0 {
            return 0.0;
        }
        
        let tf = self.calculate_tf(index, doc_id, query_terms);
        tf / doc_length
    }
    
    /// 计算自定义评分
    fn calculate_custom(
        &self,
        index: &InvertedIndex,
        doc_id: DocumentId,
        query_terms: &[String],
    ) -> f64 {
        // 默认使用 BM25
        self.calculate_bm25(index, doc_id, query_terms)
    }
    
    /// 获取词项频率
    fn get_term_frequency(
        &self,
        index: &InvertedIndex,
        term: &str,
        doc_id: DocumentId,
    ) -> f64 {
        if let Some(posting_list) = index.search_term(term) {
            if let Some(posting) = posting_list.get_posting(doc_id) {
                return posting.term_freq as f64;
            }
        }
        0.0
    }
    
    /// 获取文档长度
    fn get_doc_length(&self, index: &InvertedIndex, doc_id: DocumentId) -> f64 {
        if let Some(doc) = index.get_document(doc_id) {
            doc.all_text().split_whitespace().count() as f64
        } else {
            0.0
        }
    }
    
    /// 计算平均文档长度
    fn calculate_avg_doc_length(&self, index: &InvertedIndex) -> f64 {
        let doc_count = index.document_count();
        if doc_count == 0 {
            return 0.0;
        }
        
        let mut total_length = 0.0;
        
        // 遍历所有词项计算总长度
        for term in index.all_terms() {
            if let Some(posting_list) = index.search_term(&term) {
                for posting in posting_list.postings {
                    total_length += posting.term_freq as f64;
                }
            }
        }
        
        total_length / doc_count as f64
    }
    
    /// 获取配置
    pub fn config(&self) -> &RankerConfig {
        &self.config
    }
    
    /// 更新配置
    pub fn set_config(&mut self, config: RankerConfig) {
        self.config = config;
    }
}

impl Default for Ranker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试评分文档排序
    #[test]
    fn test_scored_document_ordering() {
        let mut docs = vec![
            ScoredDocument::new(DocumentId(1), 0.5),
            ScoredDocument::new(DocumentId(2), 0.9),
            ScoredDocument::new(DocumentId(3), 0.3),
        ];
        
        docs.sort();
        
        assert_eq!(docs[0].doc_id, DocumentId(2));
        assert_eq!(docs[1].doc_id, DocumentId(1));
        assert_eq!(docs[2].doc_id, DocumentId(3));
    }
    
    /// 测试 BM25 评分
    #[test]
    fn test_bm25_scoring() {
        let index = InvertedIndex::new();
        
        // 添加文档
        let doc1 = Document::new(DocumentId(0), "测试".to_string(), "hello world".to_string());
        let doc2 = Document::new(DocumentId(0), "测试".to_string(), "hello hello hello".to_string());
        
        index.add_document(doc1);
        index.add_document(doc2);
        
        let ranker = Ranker::new();
        
        // 搜索并排序
        let doc_ids = index.search_or(&["hello"]);
        let results = ranker.rank(&index, doc_ids, &["hello".to_string()]);
        
        assert!(!results.is_empty());
    }
    
    /// 测试排序器配置
    #[test]
    fn test_ranker_config() {
        let config = RankerConfig {
            method: ScoringMethod::TfIdf,
            bm25_k1: 1.5,
            bm25_b: 0.8,
            return_documents: false,
            max_results: 50,
        };
        
        let ranker = Ranker::with_config(config);
        
        assert_eq!(ranker.config().method, ScoringMethod::TfIdf);
        assert_eq!(ranker.config().max_results, 50);
    }
}
