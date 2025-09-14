<script lang="ts">
    import { onMount } from "svelte";

    let documents: any[] = [];
    let fileInput: HTMLInputElement;
    let description = "";
    let uploading = false;
    let uploadResponse = "";
    let isDragOver = false;

    onMount(async () => {
        const containerId = (window as any).CONTAINER_ID || "unknown";
        console.log(`Frontend Container ID: ${containerId}`);

        loadDocuments();
    });

    async function loadDocuments() {
        try {
            const response = await fetch("/api/documents");

            if (response.ok) {
                documents = await response.json();
            } else {
                console.error("Error loading documents");
            }
        } catch (error) {
            console.error("Failed to load documents:", error);
        }
    }

    async function handleUpload(event?: Event) {
        if (event) {
            event.preventDefault();
        }

        if (!fileInput.files || !fileInput.files[0]) {
            alert("Por favor, selecione um arquivo.");
            return;
        }

        uploading = true;
        uploadResponse = "";

        const formData = new FormData();
        formData.append("file", fileInput.files[0]);
        if (description.trim()) {
            formData.append("description", description.trim());
        }

        try {
            const response = await fetch("/api/documents", {
                method: "POST",
                body: formData,
            });

            const result = await response.text();

            if (response.ok) {
                const data = JSON.parse(result);
                const documentId = data.id;
                const fileName = fileInput.files![0].name;

                uploadResponse = `✅ Upload realizado com sucesso!
ID do documento: ${documentId}
Arquivo: ${fileName}
Link de acesso: ${window.location.origin}/documents/${documentId}`;

                fileInput.value = "";
                description = "";
                loadDocuments();
            } else {
                throw new Error(result || "Erro no upload");
            }
        } catch (error: any) {
            uploadResponse = `❌ Erro no upload: ${error.message}`;
        } finally {
            uploading = false;
        }
    }

    function getFileIcon(mimeType: string): string {
        if (mimeType.includes("pdf")) return "📄";
        if (mimeType.includes("image")) return "🖼️";
        if (mimeType.includes("video")) return "🎥";
        if (mimeType.includes("audio")) return "🎵";
        if (mimeType.includes("text")) return "📝";
        if (mimeType.includes("word")) return "📄";
        if (mimeType.includes("excel") || mimeType.includes("spreadsheet"))
            return "📊";
        if (
            mimeType.includes("powerpoint") ||
            mimeType.includes("presentation")
        )
            return "📊";
        return "📁";
    }

    function formatFileSize(bytes: number): string {
        if (bytes === 0) return "0 Bytes";
        const k = 1024;
        const sizes = ["Bytes", "KB", "MB", "GB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    }

    function handleDragOver(event: DragEvent) {
        event.preventDefault();
        event.stopPropagation();
        isDragOver = true;
    }

    function handleDragLeave() {
        isDragOver = false;
    }

    function handleDrop(event: DragEvent) {
        event.preventDefault();
        event.stopPropagation();
        isDragOver = false;

        const files = event.dataTransfer?.files;
        if (files && files.length > 0) {
            // Create a new FileList-like object
            const dt = new DataTransfer();
            dt.items.add(files[0]);
            fileInput.files = dt.files;
        }
    }

    function openDocument(docId: string) {
        window.open(`/api/documents/${docId}`, "_blank");
    }
</script>

<svelte:head>
    <title>CDN - Upload e Visualização de Arquivos</title>
</svelte:head>

<div class="container">
    <h1>📁 CDN - Sistema de Arquivos</h1>

    <div
        class="upload-section"
        class:drag-over={isDragOver}
        on:dragover={handleDragOver}
        on:dragleave={handleDragLeave}
        on:drop={handleDrop}
    >
        <h3>📤 Upload de Arquivo</h3>
        <form on:submit={handleUpload}>
            <div class="file-input">
                <input type="file" bind:this={fileInput} required />
            </div>
            <div>
                <input
                    type="text"
                    class="description-input"
                    bind:value={description}
                    placeholder="Descrição do arquivo (opcional)"
                    maxlength="255"
                />
            </div>
            <button type="submit" disabled={uploading}>
                {uploading ? "Enviando..." : "Fazer Upload"}
            </button>
        </form>

        {#if uploading}
            <div class="loading">
                <div class="spinner"></div>
                <p>Enviando arquivo...</p>
            </div>
        {/if}

        {#if uploadResponse}
            <div
                class="response-area"
                class:error={uploadResponse.includes("❌")}
            >
                {@html uploadResponse.replace(/\n/g, "<br>")}
            </div>
        {/if}
    </div>
</div>

<div class="container documents-container">
    <h2>📋 Documentos Enviados</h2>
    <div id="documentsList">
        {#if documents.length === 0}
            <p>Carregando documentos...</p>
        {:else}
            <div class="documents-grid">
                {#each documents as doc}
                    <div
                        class="document-card"
                        on:click={() => openDocument(doc.id)}
                        role="button"
                        tabindex="0"
                        on:keypress={(e) =>
                            e.key === "Enter" && openDocument(doc.id)}
                    >
                        <div class="document-icon">
                            {getFileIcon(doc.mime_type)}
                        </div>
                        <div class="document-info">
                            <h4>Documento</h4>
                            <p><strong>Tipo:</strong> {doc.mime_type}</p>
                            <p>
                                <strong>Tamanho:</strong>
                                {formatFileSize(doc.file_size)}
                            </p>
                            <p>
                                <strong>Criado em:</strong>
                                {new Date(doc.created_at).toLocaleString(
                                    "pt-BR",
                                )}
                            </p>
                            {#if doc.description}
                                <p class="document-description">
                                    "{doc.description}"
                                </p>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
</div>

<style>
    :global(body) {
        font-family:
            -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        max-width: 800px;
        margin: 0 auto;
        padding: 20px;
        background-color: #f5f5f5;
    }

    .container {
        background: white;
        padding: 30px;
        border-radius: 10px;
        box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
    }

    .documents-container {
        margin-top: 20px;
    }

    .instance-info {
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        color: white;
        padding: 20px;
        border-radius: 10px;
        margin-bottom: 20px;
    }

    .instance-info h2 {
        margin: 0 0 15px 0;
        text-align: center;
    }

    .info-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 15px;
    }

    .info-card {
        background: rgba(255, 255, 255, 0.1);
        padding: 15px;
        border-radius: 8px;
        text-align: center;
    }

    .info-card h3 {
        margin: 0 0 10px 0;
        font-size: 14px;
        opacity: 0.9;
    }

    .instance-id {
        font-family: "Courier New", monospace;
        font-size: 16px;
        font-weight: bold;
        margin: 0;
        word-break: break-all;
    }

    h1 {
        color: #333;
        text-align: center;
        margin-bottom: 30px;
    }

    .upload-section {
        border: 2px dashed #ddd;
        border-radius: 8px;
        padding: 30px;
        text-align: center;
        margin-bottom: 30px;
        transition: border-color 0.3s;
    }

    .upload-section:hover,
    .upload-section.drag-over {
        border-color: #007bff;
        background-color: #f8f9ff;
    }

    .file-input {
        margin: 20px 0;
    }

    input[type="file"] {
        margin-bottom: 15px;
    }

    button {
        background: #007bff;
        color: white;
        border: none;
        padding: 12px 24px;
        border-radius: 5px;
        cursor: pointer;
        font-size: 16px;
        transition: background-color 0.3s;
    }

    button:hover {
        background: #0056b3;
    }

    button:disabled {
        background: #ccc;
        cursor: not-allowed;
    }

    .description-input {
        width: 100%;
        padding: 10px;
        border: 1px solid #ddd;
        border-radius: 5px;
        margin-bottom: 15px;
        font-family: inherit;
    }

    .loading {
        text-align: center;
        margin: 20px 0;
    }

    .spinner {
        border: 3px solid #f3f3f3;
        border-top: 3px solid #007bff;
        border-radius: 50%;
        width: 30px;
        height: 30px;
        animation: spin 1s linear infinite;
        margin: 0 auto;
    }

    @keyframes spin {
        0% {
            transform: rotate(0deg);
        }
        100% {
            transform: rotate(360deg);
        }
    }

    .response-area {
        margin: 15px 0;
        padding: 10px;
        border-radius: 5px;
        background: #d4edda;
        color: #155724;
    }

    .response-area.error {
        background: #f8d7da;
        color: #721c24;
    }

    .documents-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        gap: 15px;
        margin-top: 20px;
    }

    .document-card {
        border: 1px solid #ddd;
        border-radius: 8px;
        padding: 15px;
        background: white;
        transition: box-shadow 0.2s;
        cursor: pointer;
    }

    .document-card:hover {
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }

    .document-icon {
        font-size: 48px;
        text-align: center;
        margin-bottom: 10px;
    }

    .document-info h4 {
        margin: 0 0 8px 0;
        color: #333;
        font-size: 14px;
    }

    .document-info p {
        margin: 4px 0;
        color: #666;
        font-size: 12px;
    }

    .document-description {
        font-style: italic;
        color: #888;
        margin-top: 8px;
    }

    @media (max-width: 600px) {
        .info-grid {
            grid-template-columns: 1fr;
        }

        :global(body) {
            margin: 10px;
            padding: 15px;
        }
    }
</style>
