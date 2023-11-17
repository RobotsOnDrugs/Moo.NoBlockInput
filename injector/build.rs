use embed_manifest::embed_manifest;
use embed_manifest::new_manifest;
use embed_manifest::manifest::ExecutionLevel;
use embed_manifest::manifest::ActiveCodePage;
use embed_manifest::manifest::MaxVersionTested;

fn main()
{
	let manifest_builder = new_manifest("default")
		.active_code_page(ActiveCodePage::Utf8)
		// Not actually tested on Windows 11, but close enough
		.max_version_tested(MaxVersionTested::Windows11Version22H2)
		.name("NoBlockInput Injector")
		.requested_execution_level(ExecutionLevel::RequireAdministrator)
		.version(0, 4, 0, 0);
	embed_manifest(manifest_builder).expect("Couldn't embed manifest.");
}