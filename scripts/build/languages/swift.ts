import { toPascalCase, run } from "..";
import * as fs from "fs";

export async function buildSwift(crate: string) {
  const targets = ["aarch64-apple-ios"];
  const fatTargets: Record<string, string[]> = {
    "ios-sim": ["x86_64-apple-ios", "aarch64-apple-ios-sim"],
    catalyst: ["x86_64-apple-ios-macabi", "aarch64-apple-ios-macabi"],
    macos: ["x86_64-apple-darwin", "aarch64-apple-darwin"],
  };

  let cargoBuildCmd = `cargo --color always build --manifest-path crates/${crate}_ffi/Cargo.toml --features ffi_uniffi`;

  const allTargets = [...Object.values(fatTargets).flat(), ...targets];

  allTargets.map((target) => {
    cargoBuildCmd += ` --target ${target}`;
  });

  await run(cargoBuildCmd);

  await run(
    `cargo --color always run -p uniffi-bindgen generate --no-format --library target/aarch64-apple-darwin/debug/lib${crate}_ffi.a --language swift --out-dir target/debug/swift/${crate}`,
  );

  let createXcfCmd = "xcodebuild -create-xcframework";
  targets.forEach((target) => {
    createXcfCmd += ` -library target/${target}/debug/lib${crate}_ffi.a -headers target/debug/swift/${crate}/`;
  });

  await Promise.all(
    Object.keys(fatTargets).map(async (fatTargetName) => {
      const libPaths: string[] = [];
      fatTargets[fatTargetName].forEach((target) => {
        libPaths.push(`target/${target}/debug/lib${crate}_ffi.a`);
      });

      await run(
        `lipo -create ${libPaths.join(" ")} -output target/debug/lib${crate}_ffi-${fatTargetName}.a`,
      );

      createXcfCmd += ` -library target/debug/lib${crate}_ffi-${fatTargetName}.a -headers target/debug/swift/${crate}/`;
    }),
  );

  const swiftPackage = toPascalCase(crate);
  createXcfCmd += ` -output packages/swift/${swiftPackage}/Frameworks/${crate}.xcframework`;

  if (
    fs.existsSync(
      `packages/swift/${swiftPackage}/Frameworks/${crate}.xcframework`,
    )
  ) {
    fs.rmdirSync(
      `packages/swift/${swiftPackage}/Frameworks/${crate}.xcframework`,
      {
        recursive: true,
      },
    );
  }

  // xcframework needs the modulemap to be named module.modulemap
  fs.renameSync(
    `target/debug/swift/${crate}/${crate}FFI.modulemap`,
    `target/debug/swift/${crate}/module.modulemap`,
  );

  // replace var with let to resolve swift concurrency issues
  // I believe this is fixed in https://github.com/mozilla/uniffi-rs/pull/2294
  // The above PR is available in uniffi-rs 0.29.0, but we won't be updating until
  // Nord generators (i.e. Golang) are updated to use 0.29.0
  let content = fs.readFileSync(
    `target/debug/swift/${crate}/${crate}.swift`,
    "utf-8",
  );
  content = content.replace(
    "private var initializationResult",
    "private let initializationResult",
  );

  fs.writeFileSync(`target/debug/swift/${crate}/${crate}.swift`, content);

  await run(createXcfCmd);

  fs.renameSync(
    `target/debug/swift/${crate}/${crate}.swift`,
    `packages/swift/${swiftPackage}/Sources/${swiftPackage}/${swiftPackage}.swift`,
  );

  fs.copyFileSync(
    `crates/${crate}_ffi/test_data.json`,
    `packages/swift/${swiftPackage}/Tests/AlgoModelsTests/Resources/test_data.json`,
  );

  console.log(`Updated ${swiftPackage} in packages/swift/${swiftPackage}/`);
}
