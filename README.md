# GeneaGraph: Rust Family Tree Visualizer

[![dependency status](https://deps.rs/repo/github/emilk/eframe_template/status.svg)](https://deps.rs/repo/github/emilk/eframe_template)
[![Build Status](https://github.com/emilk/eframe_template/workflows/CI/badge.svg)](https://github.com/emilk/eframe_template/actions?workflow=CI)
![WASM](https://img.shields.io/badge/compiles_to-WASM-orange?logo=webassembly)
![egui](https://img.shields.io/badge/powered_by-egui-3b82f6?style=flat&logo=rust)


**GeneaGraph** is a high-performance, web-based family tree visualization and management tool built with **Rust** and **egui**. It allows users to interactively view, create, and modify genealogical data through a node-based graph interface. The application compiles to WebAssembly (WASM) to run directly in the browser while communicating with a backend REST API.

## 🚀 Features

*   **Interactive Graph Visualization:**
    *   Visualize family members as nodes and relationships as connections.
    *   Zoom, pan, and drag nodes for better organization.
    *   Automatic hierarchical layout adjustments.
*   **Person Management:**
    *   Create new profiles with details: Name, Last Name, Birth Date, Death Date (if applicable), and Country.
    *   View detailed information by clicking on a node.
*   **Relationship Editing:**
    *   **Parent/Child:** Link two nodes to establish lineage.
    *   **Cousins:** Link two nodes to establish cousin relationships.
    *   **Deletion:** Remove nodes or relationships directly from the graph.
*   **Genealogical Queries:**
    *   **Ancestors:** Retrieve and list all ancestors of a selected person.
    *   **NCA (Nearest Common Ancestor):** Calculate and display the nearest common ancestor between two selected people.
*   **Search & Debug:**
    *   Monitor backend API events.
    *   Debug panel showing source/target IDs and raw data.

## 🛠️ Tech Stack

*   **Language:** Rust
*   **GUI Framework:** [egui](https://github.com/emilk/egui) (Immediate mode GUI)
*   **Web Framework:** [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
*   **Graphing Library:** [egui_graphs](https://github.com/blurry-mood/egui_graphs) & [petgraph](https://github.com/petgraph/petgraph)
*   **HTTP Client:** [reqwest](https://github.com/seanmonstar/reqwest) (WASM compatible)
*   **Target:** WASM (WebAssembly)

## 🎮 Controls & Shortcuts

The application relies on mouse interactions and keyboard shortcuts to manage the graph:

| Action | Control |
| :--- | :--- |
| **Select Node/Edge** | `Left Click` |
| **Move Graph** | `Middle Mouse Button` (Hold & Drag) |
| **Zoom** | `Left Ctrl` + `Scroll Wheel` |
| **Create Parent Link** | Select 2 nodes, then `Left Alt` + `J` |
| **Create Cousin Link** | Select 2 nodes, then `Left Alt` + `C` |
| **Delete Item** | Select node/edge, then `Delete` |
| **Refresh Graph** | `Left Alt` + `R` |

> **Note:** For creating connections, the direction is usually **Source ID -> Target ID**. The order in which you select nodes matters.

## 🔌 API Requirements

This frontend expects a RESTful backend running at the configured address (default: Azure). It requires the following endpoints:

*   `GET /people` - Retrieve all nodes.
*   `GET /relations` - Retrieve all edges.
*   `POST /person` - Create a new person.
*   `GET /person/{id}` - Get details for a specific person.
*   `DELETE /person/{id}` - Delete a person.
*   `POST /person/{src}/child/{tgt}` - Create a parent-child link.
*   `POST /person/{src}/cousin/{tgt}` - Create a cousin link.
*   `DELETE /person/{src}/{relation}/{tgt}` - Delete a relationship.
*   `GET /person/{id}/ancestors` - specific query endpoint.
*   `GET /person/{id}/nca/{id2}` - specific query endpoint.


## 💻 Native Execution

To run the application natively on your operating system:

1.  **Update Rust:**
    ```bash
    rustup update
    ```
2.  **Run:**
    ```bash
    cargo run --release
    ```

### Linux Dependencies
If you are on Linux, you may need specific libraries before compiling.

<details>
<summary><strong>Click to expand Ubuntu/Debian dependencies</strong></summary>

```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```
</details>

<details>
<summary><strong>Click to expand Fedora dependencies</strong></summary>

```bash
dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel
```
</details>

## 🌐 Web Development

We use [Trunk](https://trunkrs.dev/) to build and bundle the WASM application.

1.  **Add the WASM target:**
    ```bash
    rustup target add wasm32-unknown-unknown
    ```
2.  **Install Trunk:**
    ```bash
    cargo install --locked trunk
    ```
3.  **Serve Locally:**
    ```bash
    trunk serve
    ```

Once running, open `http://127.0.0.1:8080/index.html#dev`.

> **Note on Caching:** The `#dev` suffix in the URL is important during development. It bypasses the Service Worker cache (used for offline PWA support) so you always see your latest code changes immediately.

## ☁️ Deployment

To deploy the application as a static website (e.g., for GitHub Pages):

1.  **Build for Release:**
    ```bash
    trunk build --release
    ```
    This generates a `dist` directory containing the static HTML, JS, and WASM files.

2.  **Host:**
    Upload the contents of the `dist` directory to any static file host (GitHub Pages, Vercel, Netlify, etc.).

### GitHub Pages Automation
This repository includes a workflow in `.github/workflows/pages.yml`. To enable auto-deployment:
1.  Go to your Repository **Settings** -> **Pages**.
2.  Set **Source** to `gh-pages` branch and folder to `/` (root).
3.  Pushing to `main` will now automatically build and deploy to the `gh-pages` branch.
