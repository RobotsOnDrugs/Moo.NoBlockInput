use winresource::WindowsResource;

fn main()
{
	let mut res = WindowsResource::new();
	let file_description = std::env::var("BINARY_FILE_DESCRIPTION");
	if let Ok(desc) = file_description
	{
		res.set("FileDescription", desc.as_str());
		res.set("ProductName", desc.as_str());
	};
	match res.compile()
	{
		Ok(()) => {}
		Err(err) => { eprintln!("Something went wrong creating metadata for the DLL binaries! {:?}", err); panic!() }
	};
}