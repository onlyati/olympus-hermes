use std::fmt;

/// Hermes configuration sturcture
/// 
/// # Elements
/// - Persistent data store path
/// - Address of Hermes in `<hostname>:<port>` format
/// - Number of threads
pub struct HermesConfig 
{
    pub data_path: String,
    pub addr: String,
    pub threads: usize,
}

impl HermesConfig 
{
    /// Create new structure
    /// 
    /// This function initialize a new sturcture and return with it.
    /// 
    /// # Input(s)
    /// 
    /// - data: path where persistent data is stored
    /// - addr: Address of Hermes in `<hostname>:<port>` format
    /// - threads: number of threads which will be spawned by process
    /// 
    /// # Return value
    /// 
    /// HermesConfig structure.
    pub fn new(data: String, addr: String, threads: usize) -> HermesConfig 
    {
        HermesConfig 
        {
            data_path: data,
            addr: addr,
            threads: threads,
        }
    }
}

impl fmt::Debug for HermesConfig
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("HermesConfig:")
            .field("data_path", &self.data_path)
            .field("address", &self.addr)
            .field("threads", &self.threads)
            .finish()
    }
}

impl fmt::Display for HermesConfig
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("HermesConfig:")
            .field("data_path", &self.data_path)
            .field("address", &self.addr)
            .field("threads", &self.threads)
            .finish()
    }
}