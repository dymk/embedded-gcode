/// A utility function to bind a method to a target object, creating a closure that can be used as a parser.
///
/// # Arguments
///
/// * `target` - A reference to the target object.
/// * `method` - A method of the target object that takes a reference to a parameter and returns an output.
///
/// # Returns
///
/// A closure that takes a reference to a parameter and returns the output of the bound method.
pub fn bind<'p, 't, Target, Param: ?Sized, Out>(
    target: &'t Target,
    method: fn(&'t Target, &'p Param) -> Out,
) -> impl FnMut(&'p Param) -> Out + use<'p, 't, Target, Param, Out> {
    move |input| method(target, input)
}
