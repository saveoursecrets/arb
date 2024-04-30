# ARB

Command line tool to localize Flutter apps using the [DeepL][] translation API.

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

To see what changes would be made use `--dry-run` which will skip calls to the [DeepL][] API:

```
arb translate --lang fr --pro --dry-run l10n.yaml
```

For more commands and options run `arb help`.

## Notes

### Cache

Once a translation has been created the program will use a diff of the template keys to only translate when necessary and delete translations that have been removed. In order to detect changes to strings a cache file is kept in the application resource bundle directory named `.cache.arb`.

### Overrides

If you have human improvements or corrections to the machine-generated translations you can use the `--overrides` option to prefer human provided translations.

## License

MIT or Apache-2.0 at your discretion.

© Copyright Save Our Secrets Pte Ltd 2024; all rights reserved.

[DeepL]: https://deepl.com
