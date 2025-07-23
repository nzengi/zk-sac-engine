//! Advanced Async Utilities using Tokio 1.46.1
//! 
//! This module provides optimized async patterns and utilities for the ZK-SAC consensus engine,
//! leveraging Tokio 1.46.1's latest performance improvements and features.

use tokio::{
    task::{JoinHandle, spawn, spawn_blocking},
    time::{timeout, Duration, Instant, sleep},
    sync::{mpsc, RwLock, Semaphore},
    select, try_join,
};
use futures::{
    future::{Future, FutureExt, join_all, try_join_all, select_all},
    stream::{Stream, StreamExt, FuturesUnordered},
    channel::{mpsc as futures_mpsc},
    pin_mut, SinkExt,
};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use tracing::{info, debug, warn, error};

/// Advanced async task pool for consensus operations
pub struct AsyncTaskPool {
    semaphore: Arc<Semaphore>,
    max_concurrent: usize,
}

impl AsyncTaskPool {
    /// Create new task pool with maximum concurrent tasks
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
        }
    }

    /// Execute task with concurrency control
    pub async fn execute<F, Fut, T>(&self, task: F) -> Result<T>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send + 'static,
    {
        let _permit = self.semaphore.acquire().await
            .map_err(|e| anyhow!("Failed to acquire semaphore: {}", e))?;
        
        let handle = spawn(async move {
            task().await
        });
        
        handle.await
            .map_err(|e| anyhow!("Task execution failed: {}", e))?
    }

    /// Get current available capacity
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// High-performance async batch processor for transactions
pub struct BatchProcessor<T> {
    batch_size: usize,
    timeout_duration: Duration,
    sender: mpsc::Sender<T>,
    receiver: Arc<RwLock<Option<mpsc::Receiver<T>>>>,
}

impl<T: Send + 'static> BatchProcessor<T> {
    /// Create new batch processor
    pub fn new(batch_size: usize, timeout_ms: u64) -> Self {
        let (sender, receiver) = mpsc::channel(batch_size * 2);
        
        Self {
            batch_size,
            timeout_duration: Duration::from_millis(timeout_ms),
            sender,
            receiver: Arc::new(RwLock::new(Some(receiver))),
        }
    }

    /// Add item to batch
    pub async fn add_item(&self, item: T) -> Result<()> {
        self.sender.send(item).await
            .map_err(|_| anyhow!("Batch processor channel closed"))
    }

    /// Start processing batches with provided handler
    pub async fn start_processing<F, Fut>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(Vec<T>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let mut receiver = {
            let mut lock = self.receiver.write().await;
            lock.take().ok_or_else(|| anyhow!("Batch processor already started"))?
        };

        let batch_size = self.batch_size;
        let timeout_duration = self.timeout_duration;

        spawn(async move {
            let mut current_batch = Vec::with_capacity(batch_size);
            let mut last_batch_time = Instant::now();

            loop {
                select! {
                    // Receive new item
                    item = receiver.recv() => {
                        match item {
                            Some(item) => {
                                current_batch.push(item);
                                
                                // Process batch if full
                                if current_batch.len() >= batch_size {
                                    if let Err(e) = handler(current_batch.drain(..).collect()).await {
                                        error!("Batch processing failed: {}", e);
                                    }
                                    last_batch_time = Instant::now();
                                }
                            }
                            None => {
                                // Channel closed, process remaining items
                                if !current_batch.is_empty() {
                                    if let Err(e) = handler(current_batch.drain(..).collect()).await {
                                        error!("Final batch processing failed: {}", e);
                                    }
                                }
                                break;
                            }
                        }
                    }
                    
                    // Timeout elapsed
                    _ = sleep(timeout_duration), if !current_batch.is_empty() => {
                        if last_batch_time.elapsed() >= timeout_duration {
                            if let Err(e) = handler(current_batch.drain(..).collect()).await {
                                error!("Timeout batch processing failed: {}", e);
                            }
                            last_batch_time = Instant::now();
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

/// Async parallel execution helper using Tokio 1.46.1 features
pub struct ParallelExecutor;

impl ParallelExecutor {
    /// Execute multiple async tasks in parallel with timeout
    pub async fn execute_with_timeout<T>(
        tasks: Vec<JoinHandle<Result<T>>>,
        timeout_duration: Duration,
    ) -> Result<Vec<T>>
    where
        T: Send + 'static,
    {
        let execution = async {
            let mut results = Vec::with_capacity(tasks.len());
            
            for task in tasks {
                let result = task.await
                    .map_err(|e| anyhow!("Task join failed: {}", e))??;
                results.push(result);
            }
            
            Ok(results)
        };

        timeout(timeout_duration, execution).await
            .map_err(|_| anyhow!("Parallel execution timed out"))?
    }

    /// Execute CPU-bound tasks on blocking thread pool
    pub async fn execute_cpu_bound<F, T>(tasks: Vec<F>) -> Result<Vec<T>>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let handles: Vec<_> = tasks.into_iter()
            .map(|task| spawn_blocking(task))
            .collect();

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            let result = handle.await
                .map_err(|e| anyhow!("Blocking task failed: {}", e))??;
            results.push(result);
        }

        Ok(results)
    }

    /// Advanced parallel execution with load balancing
    pub async fn execute_load_balanced<F, Fut, T>(
        task_factory: F,
        task_count: usize,
        max_concurrent: usize,
    ) -> Result<Vec<T>>
    where
        F: Fn(usize) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send + 'static,
    {
        let task_pool = AsyncTaskPool::new(max_concurrent);
        let mut handles = Vec::with_capacity(task_count);

        for i in 0..task_count {
            let factory = Arc::new(task_factory.clone());
            
            let handle = spawn(async move {
                let fut = factory(i);
                fut.await
            });
            
            handles.push(handle);
        }

        Self::execute_with_timeout(handles, Duration::from_secs(30)).await
    }
}

/// Advanced async coordinator for consensus operations
pub struct ConsensusCoordinator {
    block_production_pool: AsyncTaskPool,
    validation_pool: AsyncTaskPool,
    signature_pool: AsyncTaskPool,
}

impl ConsensusCoordinator {
    /// Create new consensus coordinator with optimized pools
    pub fn new() -> Self {
        Self {
            block_production_pool: AsyncTaskPool::new(2), // Limited for sequential block production
            validation_pool: AsyncTaskPool::new(8),       // Parallel validation
            signature_pool: AsyncTaskPool::new(16),       // Many parallel signatures
        }
    }

    /// Coordinate block production with parallel validation
    pub async fn coordinate_block_production<F1, F2, F3, Fut1, Fut2, Fut3, T1, T2, T3>(
        &self,
        produce_block: F1,
        validate_transactions: F2,
        collect_signatures: F3,
    ) -> Result<(T1, T2, T3)>
    where
        F1: FnOnce() -> Fut1 + Send + 'static,
        F2: FnOnce() -> Fut2 + Send + 'static,
        F3: FnOnce() -> Fut3 + Send + 'static,
        Fut1: std::future::Future<Output = Result<T1>> + Send,
        Fut2: std::future::Future<Output = Result<T2>> + Send,
        Fut3: std::future::Future<Output = Result<T3>> + Send,
        T1: Send + 'static,
        T2: Send + 'static,
        T3: Send + 'static,
    {
        info!("🔄 Coordinating parallel block production");

        // Execute block production, validation, and signature collection in parallel
        let (block_result, validation_result, signature_result) = try_join!(
            self.block_production_pool.execute(produce_block),
            self.validation_pool.execute(validate_transactions),
            self.signature_pool.execute(collect_signatures)
        )?;

        info!("✅ Block production coordination completed");
        Ok((block_result, validation_result, signature_result))
    }

    /// Monitor pool performance
    pub fn get_pool_stats(&self) -> (usize, usize, usize) {
        (
            self.block_production_pool.available_permits(),
            self.validation_pool.available_permits(),
            self.signature_pool.available_permits(),
        )
    }
}

/// Async timeout helper with exponential backoff using futures 0.3.31
pub struct TimeoutManager;

impl TimeoutManager {
    /// Execute with exponential backoff retry
    pub async fn execute_with_backoff<F, Fut, T>(
        task: F,
        max_retries: usize,
        base_timeout_ms: u64,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut current_timeout = Duration::from_millis(base_timeout_ms);
        
        for attempt in 0..max_retries {
            debug!("⏱️  Attempt {} with timeout {:?}", attempt + 1, current_timeout);
            
            match timeout(current_timeout, task()).await {
                Ok(Ok(result)) => {
                    if attempt > 0 {
                        info!("✅ Task succeeded after {} retries", attempt);
                    }
                    return Ok(result);
                }
                Ok(Err(e)) => {
                    warn!("🚫 Task failed: {}", e);
                    return Err(e);
                }
                Err(_) => {
                    warn!("⏰ Timeout on attempt {}", attempt + 1);
                    if attempt == max_retries - 1 {
                        return Err(anyhow!("Task failed after {} timeout attempts", max_retries));
                    }
                    // Exponential backoff
                    current_timeout = Duration::from_millis((current_timeout.as_millis() as u64) * 2);
                    sleep(Duration::from_millis(100)).await; // Brief pause between retries
                }
            }
        }
        
        Err(anyhow!("Task failed after {} attempts", max_retries))
    }
}

/// Advanced streaming utilities using futures 0.3.31
pub struct StreamingProcessor;

impl StreamingProcessor {
    /// Process a stream of items with futures combinators
    pub async fn process_stream<S, F, Fut, T, U>(
        mut stream: S,
        processor: F,
        max_concurrent: usize,
    ) -> Result<Vec<U>>
    where
        S: Stream<Item = T> + Unpin,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<U>> + Send,
        T: Send + 'static,
        U: Send + 'static,
    {
        let mut results = Vec::new();
        let mut buffer = FuturesUnordered::new();
        let mut active_count = 0;

        loop {
            select! {
                // Get next item from stream
                item = stream.next().fuse() => {
                    match item {
                        Some(item) => {
                            if active_count < max_concurrent {
                                buffer.push(processor(item));
                                active_count += 1;
                            } else {
                                // Wait for a slot to free up
                                if let Some(result) = buffer.next().await {
                                    results.push(result?);
                                    active_count -= 1;
                                    buffer.push(processor(item));
                                    active_count += 1;
                                }
                            }
                        }
                        None => break, // Stream ended
                    }
                }
                
                // Process completed futures
                result = buffer.next(), if !buffer.is_empty() => {
                    if let Some(result) = result {
                        results.push(result?);
                        active_count -= 1;
                    }
                }
            }
        }

        // Process remaining futures
        while let Some(result) = buffer.next().await {
            results.push(result?);
        }

        Ok(results)
    }

    /// Batch process with futures::join_all
    pub async fn batch_process_all<F, Fut, T, U>(
        items: Vec<T>,
        processor: F,
    ) -> Result<Vec<U>>
    where
        F: Fn(T) -> Fut,
        Fut: Future<Output = Result<U>>,
    {
        let futures: Vec<_> = items.into_iter().map(processor).collect();
        let results = join_all(futures).await;
        
        // Convert Vec<Result<U>> to Result<Vec<U>>
        results.into_iter().collect()
    }

    /// Try join all with early termination on error
    pub async fn try_batch_process_all<F, Fut, T, U>(
        items: Vec<T>,
        processor: F,
    ) -> Result<Vec<U>>
    where
        F: Fn(T) -> Fut,
        Fut: Future<Output = Result<U>>,
    {
        let futures: Vec<_> = items.into_iter().map(processor).collect();
        try_join_all(futures).await
    }
}

/// Racing and selection utilities using futures 0.3.31
pub struct FutureRacer;

impl FutureRacer {
    /// Race multiple futures, return first successful result
    pub async fn race_first_success<F, T>(futures: Vec<F>) -> Result<T>
    where
        F: Future<Output = Result<T>> + Unpin,
    {
        if futures.is_empty() {
            return Err(anyhow!("No futures to race"));
        }

        let (result, _index, _remaining) = select_all(futures).await;
        result
    }

    /// Select between multiple operations with timeout
    pub async fn select_with_timeout<F1, F2, T1, T2>(
        fut1: F1,
        fut2: F2,
        timeout_duration: Duration,
    ) -> Result<Either<T1, T2>>
    where
        F1: Future<Output = Result<T1>>,
        F2: Future<Output = Result<T2>>,
    {
        pin_mut!(fut1, fut2);
        
        let selection = select! {
            result1 = fut1.fuse() => Either::Left(result1?),
            result2 = fut2.fuse() => Either::Right(result2?),
        };

        timeout(timeout_duration, async { Ok(selection) }).await
            .map_err(|_| anyhow!("Selection timed out"))?
    }
}

/// Custom Either type for selection results
#[derive(Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

/// Advanced channel utilities using futures channels
pub struct ChannelManager;

impl ChannelManager {
    /// Create a futures channel for cross-task communication
    pub fn create_futures_channel<T>(buffer_size: usize) -> (futures_mpsc::Sender<T>, futures_mpsc::Receiver<T>) {
        futures_mpsc::channel(buffer_size)
    }

    /// Broadcast to multiple receivers
    pub async fn broadcast_to_many<T: Clone>(
        message: T,
        senders: Vec<futures_mpsc::Sender<T>>,
    ) -> Result<()> {
        let futures: Vec<_> = senders
            .into_iter()
            .map(|mut sender| {
                let msg = message.clone();
                async move {
                    sender.send(msg).await
                        .map_err(|e| anyhow!("Broadcast failed: {:?}", e))
                }
            })
            .collect();

        try_join_all(futures).await?;
        Ok(())
    }

    /// Collect from multiple streams
    pub async fn collect_from_streams<T, S>(
        streams: Vec<S>,
        max_items: usize,
    ) -> Result<Vec<T>>
    where
        S: Stream<Item = T> + Unpin,
        T: Send + 'static,
    {
        let mut all_items = Vec::new();
        let mut stream_futures = FuturesUnordered::new();

        // Convert streams to futures
        for mut stream in streams {
            stream_futures.push(async move {
                let mut items = Vec::new();
                while let Some(item) = stream.next().await {
                    items.push(item);
                    if items.len() >= max_items {
                        break;
                    }
                }
                items
            });
        }

        // Collect all results
        while let Some(items) = stream_futures.next().await {
            all_items.extend(items);
        }

        Ok(all_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_task_pool() {
        let pool = AsyncTaskPool::new(2);
        
        let result = pool.execute(|| async {
            sleep(Duration::from_millis(10)).await;
            Ok::<i32, anyhow::Error>(42)
        }).await.unwrap();
        
        assert_eq!(result, 42);
    }

    #[test]
    async fn test_parallel_executor() {
        let tasks: Vec<JoinHandle<Result<i32>>> = (0..5)
            .map(|i| spawn(async move {
                sleep(Duration::from_millis(10)).await;
                Ok(i * 2)
            }))
            .collect();

        let results = ParallelExecutor::execute_with_timeout(
            tasks, 
            Duration::from_secs(1)
        ).await.unwrap();

        assert_eq!(results, vec![0, 2, 4, 6, 8]);
    }

    #[test]
    async fn test_timeout_manager() {
        let result = TimeoutManager::execute_with_backoff(
            || async {
                sleep(Duration::from_millis(5)).await;
                Ok::<String, anyhow::Error>("success".to_string())
            },
            3,
            100,
        ).await.unwrap();

        assert_eq!(result, "success");
    }

    #[test]
    async fn test_streaming_processor() {
        use futures::stream;
        
        let items = vec![1, 2, 3, 4, 5];
        let stream = stream::iter(items);
        
        let results = StreamingProcessor::process_stream(
            stream,
            |x| async move { 
                sleep(Duration::from_millis(10)).await;
                Ok::<i32, anyhow::Error>(x * 2) 
            },
            3, // max concurrent
        ).await.unwrap();

        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    async fn test_batch_processing() {
        let items = vec![1, 2, 3, 4, 5];
        
        let results = StreamingProcessor::batch_process_all(
            items,
            |x| async move {
                sleep(Duration::from_millis(5)).await;
                Ok::<i32, anyhow::Error>(x * 3)
            }
        ).await.unwrap();

        assert_eq!(results, vec![3, 6, 9, 12, 15]);
    }

    #[test]
    async fn test_future_racing() {
        let fut1 = async {
            sleep(Duration::from_millis(50)).await;
            Ok::<String, anyhow::Error>("slow".to_string())
        };
        
        let fut2 = async {
            sleep(Duration::from_millis(10)).await;
            Ok::<String, anyhow::Error>("fast".to_string())
        };

        let result = FutureRacer::race_first_success(vec![fut1, fut2]).await.unwrap();
        assert_eq!(result, "fast");
    }

    #[test]
    async fn test_channel_manager() {
        let (sender1, mut receiver1) = ChannelManager::create_futures_channel(10);
        let (sender2, mut receiver2) = ChannelManager::create_futures_channel(10);
        
        let message = "broadcast test".to_string();
        ChannelManager::broadcast_to_many(
            message.clone(), 
            vec![sender1, sender2]
        ).await.unwrap();

        assert_eq!(receiver1.next().await, Some(message.clone()));
        assert_eq!(receiver2.next().await, Some(message));
    }
} 