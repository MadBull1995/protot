use crate::internal::sylklabs::scheduler::v1::ExecuteRequest;

/// A trait representing a function that can be called from a box.
///
/// This is often used to store different types of closures
/// in a `Box` so they can be executed later.
pub trait FnBox<T> {
    /// Call the function from the box.
    ///
    /// Consumes the box in the process.
    fn call_box(self: Box<Self>, args: T);
}

/// Implementation of the `FnBox` trait for types that implement `FnOnce`.
///
/// This allows any type that can be called once (`FnOnce`) to be used where
/// an `FnBox` is expected.
impl<F, A> FnBox<A> for F
where
    F: FnOnce(A),
{
    fn call_box(self: Box<Self>, args: A) {
        (*self)(args)
    }
}

/// A type alias for storing boxed function objects that implement `FnBox` and `Send`.
///
/// `'a` is the lifetime for which the function object is valid.
pub type Thunk<'a, A> = Box<dyn FnBox<A> + Send + 'a>;

/// Represents a job that the thread pool will execute.
///
/// Contains a unique identifier and the job as a `Thunk`.
pub struct Job<'a, A> {
    /// A unique identifier for the job.
    pub(crate) id: usize,
    /// The actual job as a boxed function object.
    pub(crate) job: Thunk<'a, A>,
    pub(crate) data: ExecuteRequest,
}

impl Job<'_, ExecuteRequest> {
    /// Executes the job with the given arguments.
    ///
    /// Consumes the Job in the process.
    pub fn execute_job(self) {
        self.job.call_box(self.data);
    }

    /// Returns the unique identifier for the job.
    pub fn get_id(&self) -> usize {
        self.id
    }
}

// pub struct Job<'a, A, F>
// where
//     F: FnOnce(A) + Send + 'a,
// {
//     /// A unique identifier for the job.
//     pub(crate) id: usize,
//     /// The actual job as a boxed closure that takes arguments.
//     pub(crate) job: Thunk<'a, A>,
// }

// impl<'a, A, F> Job<'a, A, F>
// where
//     F: FnOnce(A) + Send + 'a,
// {
//     // ... other methods and implementation here
// }
