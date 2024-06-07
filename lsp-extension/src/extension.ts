/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import { workspace, ExtensionContext } from 'vscode';
import * as vscode from 'vscode';
import { Wasm, ProcessOptions } from '@vscode/wasm-wasi';
import { startServer, createStdioOptions } from '@vscode/wasm-wasi-lsp';

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	DidChangeConfigurationNotification,
} from 'vscode-languageclient/node';

let client: LanguageClient;
const channel = vscode.window.createOutputChannel('Ksp Config Lsp Server');

export async function activate(context: ExtensionContext) {

	const wasm: Wasm = await Wasm.load();

	const serverOptions: ServerOptions = async () => {
		const options: ProcessOptions = {
			stdio: createStdioOptions(),
			trace: true,
			mountPoints: [
				{ kind: 'workspaceFolder' },
			]
		};
		const filename = vscode.Uri.joinPath(context.extensionUri, 'server', 'ksp-cfg-lsp.wasm');
		const bits = await vscode.workspace.fs.readFile(filename);
		const module = await WebAssembly.compile(bits);
		const memory = new WebAssembly.Memory({ initial: 21, maximum: 21, shared: true});
		const process = await wasm.createProcess('ksp-cfg-lsp', module, memory, options);

		const decoder = new TextDecoder('utf-8');
		process.stderr.onData((data) => {
			channel.append(decoder.decode(data));
		});

		return startServer(process);
	};


	// Options to control the language client
	const clientOptions: LanguageClientOptions = {
		documentSelector: [{ language: 'ksp-cfg' }],
		diagnosticPullOptions: { onSave: true },
		traceOutputChannel: channel,
		outputChannel: channel
	};

	// Create the language client and start the client.
	client = new LanguageClient(
		'KspCfgLspServer',
		'Ksp Config Lsp Client',
		serverOptions,
		clientOptions
	);

	// Start the client. This will also launch the server
	client.start();
	workspace.onDidChangeConfiguration(
		async (_) => await client.sendNotification(DidChangeConfigurationNotification.type, { settings: "" })
	);
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
