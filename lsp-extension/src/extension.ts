/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	DidChangeConfigurationNotification,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {

	const serverOptions: ServerOptions = {
		command: path.join(__dirname, '../server/lsp-rs.exe'),
	};

	// Options to control the language client
	const clientOptions: LanguageClientOptions = {
		documentSelector: [{ language: 'ksp-cfg' }]
	};

	// Create the language client and start the client.
	client = new LanguageClient(
		'KspCfgLspServer',
		'Ksp Config Lsp Server',
		serverOptions,
		clientOptions
	);

	// Start the client. This will also launch the server
	client.start();
	// console.log(__dirname);
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
