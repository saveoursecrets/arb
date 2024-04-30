# ARB

Command line tool to localize Flutter apps using the DeepL translation API.

## Install

```
cargo install arb
```

## Test

Use a Free API key for the tests:

```
export DEEPL_API_KEY="<api key>"
cargo test
```

## Usage

Convert all the strings from the template language into French and write the translations to `app_fr.arb`:

```
export DEEPL_API_KEY="<api key>"
arb translate --lang fr --pro --write l10n.yaml
```

## License

MIT or Apache-2.0 at your discretion.

© Copyright Save Our Secrets Pte Ltd 2024; all rights reserved.
