use anyhow::Result;
use std::{
    env::temp_dir,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tokio::process::Command;

pub async fn ensure_roslyn_is_installed(
    version: &str,
    remove_old_server_versions: bool,
) -> Result<PathBuf> {
    let mut roslyn_server_dir = home::home_dir().expect("Unable to find home directory");
    roslyn_server_dir.push(".roslyn");
    roslyn_server_dir.push("server");

    let mut dll_version_dir = roslyn_server_dir.clone();
    dll_version_dir.push(version);

    let mut dll_path = dll_version_dir.clone();
    dll_path.push("Microsoft.CodeAnalysis.LanguageServer.dll");

    if std::path::Path::new(&dll_path).exists() {
        return Ok(dll_path);
    }

    fs_extra::dir::create_all(&roslyn_server_dir, remove_old_server_versions)?;
    fs_extra::dir::create_all(&dll_version_dir, true)?;

    let mut temp_build_dir = temp_dir();
    temp_build_dir.push("roslyn");
    fs_extra::dir::create(&temp_build_dir, true)?;

    create_csharp_project(&temp_build_dir)?;

    Command::new("dotnet")
        .arg("add")
        .arg("package")
        .arg("Microsoft.CodeAnalysis.LanguageServer.neutral")
        .arg("-v")
        .arg(version)
        .current_dir(fs::canonicalize(temp_build_dir.clone())?)
        .output()
        .await?;

    temp_build_dir.push("out");
    temp_build_dir.push("microsoft.codeanalysis.languageserver.neutral");
    temp_build_dir.push(version);
    temp_build_dir.push("content");
    temp_build_dir.push("LanguageServer");
    temp_build_dir.push("neutral");

    let copy_options = fs_extra::dir::CopyOptions::default()
        .overwrite(true)
        .content_only(true);

    fs_extra::dir::move_dir(&temp_build_dir, &dll_version_dir, &copy_options)?;
    fs_extra::dir::remove(temp_build_dir)?;

    Ok(dll_path)
}

fn create_csharp_project(temp_dir: &Path) -> Result<()> {
    let mut nuget_config_file = std::fs::File::create(temp_dir.join("NuGet.config"))?;
    nuget_config_file.write_all(NUGET.as_bytes())?;

    let mut csproj_file = std::fs::File::create(temp_dir.join("ServerDownload.csproj")).unwrap();
    csproj_file.write_all(CSPROJ.as_bytes())?;

    Ok(())
}

const NUGET: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>
<configuration>
  <packageSources>
    <clear />

    <add key=\"vs-impl\" value=\"https://pkgs.dev.azure.com/azure-public/vside/_packaging/vs-impl/nuget/v3/index.json\" />

  </packageSources>
</configuration>
    ";

const CSPROJ: &str = "<Project Sdk=\"Microsoft.NET.Sdk\">
    <PropertyGroup>
        <RestorePackagesPath>out</RestorePackagesPath>
        <TargetFramework>net8.0</TargetFramework>
        <DisableImplicitNuGetFallbackFolder>true</DisableImplicitNuGetFallbackFolder>
        <AutomaticallyUseReferenceAssemblyPackages>false</AutomaticallyUseReferenceAssemblyPackages>
    </PropertyGroup>
</Project>
";
