# GitHub Action: List Variants

This is used by Bottlerocket to dynamically get a list of all variants in the repo.
By default, this will return anything defined under the `variants` directory.
There is also an optional input flag to `filter-variants` and only get those variants that appear to be impacted by the changes in the pull request.

## Workflow Config Example

```yaml
jobs:
  list-variants:
    name: "Determine variants"
    runs-on: ubuntu-latest
    outputs:
      variants: ${{ steps.get-variants.outputs.variants }}
      aarch-enemies: ${{ steps.get-variants.outputs.aarch-enemies }}
    steps:
      - uses: actions/checkout@v4
      - uses: ./.github/actions/list-variants
        id: get-variants
        with:
          filter-variants: true
          token: ${{ secrets.GITHUB_TOKEN }}
 ``` 

 ### Inputs

 * **`filter-variants`**: Whether to filter variant list based on changes.
 * **`token`**: [The `GITHUB_TOKEN` secret](https://help.github.com/en/actions/configuring-and-managing-workflows/authenticating-with-the-github_token).

 ### Outputs

 * **`variants`**: The list of variants that can be used in the test matrix.
 * **`aarch-enemies`**: A JSON blob to be used in the matrix filter to skip the variants that we don't support for aarch64.

 ## Development

 Any updates to the JavaScript action should be made in the `*.js` files in the root of the action.
 Once the necessary changes are made, use `npm run lint` to perform linting on the source code.
 Any warnings or errors from the `dist/` folder can be ignored.
 
 Once all linting issues are addressed, run `npm run prepare` to have the `dist/` content regenerated using the current code.
 This is the prepared action code that is actually run when submitting a PR.

