/// A trait representing a function that can be called from a box.
///
/// This is often used to store different types of closures
/// in a `Box` so they can be executed later.
pub trait FnBox {
    /// Call the function from the box.
    ///
    /// Consumes the box in the process.
    fn call_box(self: Box<Self>);
}

/// Implementation of the `FnBox` trait for types that implement `FnOnce`.
///
/// This allows any type that can be called once (`FnOnce`) to be used where
/// an `FnBox` is expected.
impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

/// A type alias for storing boxed function objects that implement `FnBox` and `Send`.
///
/// `'a` is the lifetime for which the function object is valid.
pub type Thunk<'a> = Box<dyn FnBox + Send + 'a>;

/// Represents a job that the thread pool will execute.
///
/// Contains a unique identifier and the job as a `Thunk`.
pub struct Job<'a> {
    /// A unique identifier for the job.
    pub(crate) id: usize,
    /// The actual job as a boxed function object.
    pub(crate) job: Thunk<'a>,
}

impl Job<'_> {
    /// Executes the job.
    ///
    /// Consumes the Job in the process.
    pub fn excute_job(self) {
        self.job.call_box()
    }

    /// Returns the unique identifier for the job.
    pub fn get_id(&self) -> usize {
        self.id
    }
}
