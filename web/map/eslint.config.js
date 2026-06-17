import js from "@eslint/js";
import tseslint from "typescript-eslint";
import globals from "globals";

export default tseslint.config(
  { ignores: ["dist/", "node_modules/", "vendor/", "*.js", "vite.config.ts", "vitest.config.ts"] },
  { languageOptions: { globals: globals.browser } },
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
  { languageOptions: { parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname } } },
  {
    rules: {
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-unsafe-argument": "error",
      "@typescript-eslint/no-unsafe-assignment": "error",
      "@typescript-eslint/no-unsafe-call": "error",
      "@typescript-eslint/no-unsafe-member-access": "error",
      "@typescript-eslint/no-unsafe-return": "error",
      "@typescript-eslint/no-floating-promises": "error",
      "@typescript-eslint/no-misused-promises": "error",
      "@typescript-eslint/switch-exhaustiveness-check": "error",
      "@typescript-eslint/strict-boolean-expressions": ["error", { allowString: false, allowNumber: false, allowNullableObject: false }],
      "@typescript-eslint/consistent-type-imports": ["error", { prefer: "type-imports", fixStyle: "separate-type-imports" }],
      "@typescript-eslint/await-thenable": "error",
      "@typescript-eslint/require-await": "error",
      "@typescript-eslint/return-await": "error",
      "no-restricted-syntax": ["error",
        { selector: "JSXElement", message: "No JSX" },
        { selector: "AssignmentExpression[operator='='] > MemberExpression[property.name='innerHTML']", message: "No raw innerHTML assignment" },
        { selector: "AssignmentExpression[operator='='] > MemberExpression[property.name='outerHTML']", message: "No raw outerHTML assignment" },
        { selector: "CallExpression[callee.property.name='insertAdjacentHTML']", message: "No insertAdjacentHTML" },
        { selector: "CallExpression[callee.name='eval']", message: "No eval" },
        { selector: "NewExpression[callee.name='Function']", message: "No new Function" },
        {
          selector: "AssignmentExpression[operator='='][left.type='MemberExpression'][left.object.name='window']",
          message: "No direct window.* assignment — use explicit exports"
        }
      ],
      "no-restricted-globals": ["error", { name: "event", message: "Use the event parameter, not the global" }],
    }
  },
  {
    files: ["src/render.ts", "src/concept-map.ts"],
    rules: {
      "no-restricted-syntax": "off",
    }
  }
);
