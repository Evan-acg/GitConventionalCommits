use std::fmt::Debug;

/// 配置源（Strategy）：所有配置来源实现统一接口
pub trait ConfigSource<T> {
    fn load(&self) -> Option<T>;
}

/// 可叠加合并的配置数据：高优先级配置覆盖低优先级同名字段
pub trait Mergeable {
    fn merge_from(&mut self, higher: Self);
}

/// 配置链（Chain of Responsibility）：按优先级排列的配置源集合，
/// 索引 0 为最高优先级。
///
/// - `first()`：短路链，返回第一个非空源的配置
/// - `merge()`：叠加链，逐字段用高优先级源的取值覆盖
pub struct ConfigChain<T> {
    sources: Vec<Box<dyn ConfigSource<T>>>,
}

impl<T> ConfigChain<T> {
    pub fn new(sources: Vec<Box<dyn ConfigSource<T>>>) -> Self {
        Self { sources }
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}

impl<T> ConfigChain<T> {
    pub fn first(&self) -> Option<T> {
        self.sources.iter().find_map(|s| s.load())
    }
}

impl<T: Default + Mergeable> ConfigChain<T> {
    pub fn merge(&self) -> Option<T> {
        let mut acc = T::default();
        let mut any = false;
        for source in self.sources.iter().rev() {
            if let Some(part) = source.load() {
                acc.merge_from(part);
                any = true;
            }
        }
        any.then_some(acc)
    }
}

impl<T: Debug> Debug for ConfigChain<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigChain")
            .field("source_count", &self.sources.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeSource(Option<i32>);
    impl ConfigSource<i32> for FakeSource {
        fn load(&self) -> Option<i32> {
            self.0
        }
    }

    #[derive(Default, PartialEq, Debug, Clone)]
    struct Point {
        x: Option<i32>,
        y: Option<i32>,
    }

    impl Mergeable for Point {
        fn merge_from(&mut self, higher: Self) {
            if higher.x.is_some() {
                self.x = higher.x;
            }
            if higher.y.is_some() {
                self.y = higher.y;
            }
        }
    }

    struct PointSource(Point);
    impl ConfigSource<Point> for PointSource {
        fn load(&self) -> Option<Point> {
            Some(self.0.clone())
        }
    }

    #[test]
    fn first_returns_highest_priority_non_empty() {
        let chain = ConfigChain::new(vec![
            Box::new(FakeSource(None)),
            Box::new(FakeSource(Some(2))),
            Box::new(FakeSource(Some(3))),
        ]);
        assert_eq!(chain.first(), Some(2));
    }

    #[test]
    fn first_empty_chain_returns_none() {
        let chain = ConfigChain::<i32>::new(vec![]);
        assert_eq!(chain.first(), None);
    }

    #[test]
    fn first_all_none_returns_none() {
        let chain = ConfigChain::new(vec![
            Box::new(FakeSource(None)),
            Box::new(FakeSource(None)),
        ]);
        assert_eq!(chain.first(), None);
    }

    #[test]
    fn merge_high_priority_fields_override_low() {
        let chain = ConfigChain::new(vec![
            Box::new(PointSource(Point { x: Some(1), y: None })),
            Box::new(PointSource(Point { x: None, y: Some(2) })),
        ]);
        let merged = chain.merge().unwrap();
        assert_eq!(merged, Point { x: Some(1), y: Some(2) });
    }

    #[test]
    fn merge_lower_source_does_not_clobber_higher() {
        let chain = ConfigChain::new(vec![
            Box::new(PointSource(Point { x: Some(1), y: Some(1) })),
            Box::new(PointSource(Point { x: Some(2), y: Some(2) })),
        ]);
        let merged = chain.merge().unwrap();
        assert_eq!(merged, Point { x: Some(1), y: Some(1) });
    }

    #[test]
    fn merge_empty_chain_returns_none() {
        let chain = ConfigChain::<Point>::new(vec![]);
        assert_eq!(chain.merge(), None);
    }
}
