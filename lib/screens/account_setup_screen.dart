import 'package:flutter/material.dart' hide ThemeData;
import 'package:shadcn_ui/shadcn_ui.dart';
import 'package:provider/provider.dart';
import 'package:flutter_svg/flutter_svg.dart';
import '../providers/auth_provider.dart';
import '../models/email_account.dart';

class AccountSetupScreen extends StatefulWidget {
  const AccountSetupScreen({super.key});

  @override
  State<AccountSetupScreen> createState() => _AccountSetupScreenState();
}

class _AccountSetupScreenState extends State<AccountSetupScreen> {
  final _formKey = GlobalKey<FormState>();
  final _emailController = TextEditingController();
  final _passwordController = TextEditingController();

  EmailProvider _selectedProvider = EmailProvider.gmail;
  bool _showPassword = false;
  bool _useCustomSettings = false;

  final _customImapHostController = TextEditingController();
  final _customImapPortController = TextEditingController(text: '993');
  final _customSmtpHostController = TextEditingController();
  final _customSmtpPortController = TextEditingController(text: '587');

  bool _customImapSSL = true;
  bool _customSmtpTLS = true;

  @override
  void dispose() {
    _emailController.dispose();
    _passwordController.dispose();
    _customImapHostController.dispose();
    _customImapPortController.dispose();
    _customSmtpHostController.dispose();
    _customSmtpPortController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: ShadTheme.of(context).colorScheme.background,
      appBar: AppBar(
        title: const Text('Add Email Account'),
        backgroundColor: ShadTheme.of(context).colorScheme.background,
        foregroundColor: ShadTheme.of(context).colorScheme.foreground,
        elevation: 0,
        leading: IconButton(
          onPressed: () => Navigator.of(context).pop(),
          icon: const Icon(LucideIcons.arrowLeft),
        ),
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Form(
            key: _formKey,
            child: Column(
              spacing: 16,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                // Provider selection
                _buildProviderSelection(),

                // const SizedBox(height: 24),

                // Email input
                ShadInputFormField(
                  controller: _emailController,
                  placeholder: const Text('Enter your email address'),
                  keyboardType: TextInputType.emailAddress,
                  validator: _validateEmail,
                ),

                // Password input
                ShadInputFormField(
                  controller: _passwordController,
                  placeholder: const Text(
                    'Enter your password or app password',
                  ),
                  obscureText: !_showPassword,
                  validator: _validatePassword,
                ),

                // Custom settings toggle
                ShadSwitchFormField(
                  initialValue: _useCustomSettings,
                  label: const Text('Use custom server settings'),
                  onChanged: (value) =>
                      setState(() => _useCustomSettings = value),
                ),

                if (_useCustomSettings) ...[
                  const SizedBox(height: 24),
                  _buildCustomSettings(),
                ],

                const Spacer(),

                // Error display
                Consumer<AuthProvider>(
                  builder: (context, authProvider, child) {
                    if (authProvider.error != null) {
                      return Container(
                        margin: const EdgeInsets.only(bottom: 16),
                        padding: const EdgeInsets.all(12),
                        decoration: BoxDecoration(
                          color: ShadTheme.of(
                            context,
                          ).colorScheme.destructive.withOpacity(0.1),
                          borderRadius: BorderRadius.circular(8),
                          border: Border.all(
                            color: ShadTheme.of(
                              context,
                            ).colorScheme.destructive.withOpacity(0.3),
                          ),
                        ),
                        child: Text(
                          authProvider.error!,
                          style: ShadTheme.of(context).textTheme.small.copyWith(
                                color: ShadTheme.of(
                                  context,
                                ).colorScheme.destructive,
                              ),
                        ),
                      );
                    }
                    return const SizedBox.shrink();
                  },
                ),

                // Continue button
                Consumer<AuthProvider>(
                  builder: (context, authProvider, child) {
                    return ShadButton(
                      onPressed:
                          authProvider.isLoading ? null : _handleContinue,
                      width: 480,
                      child: authProvider.isLoading
                          ? const SizedBox(
                              width: 16,
                              height: 16,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : const Text('Add Account'),
                    );
                  },
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildProviderSelection() {
    return ShadCard(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          spacing: 12,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Email Provider',
              style: ShadTheme.of(
                context,
              ).textTheme.large.copyWith(fontWeight: FontWeight.w600),
            ),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: EmailProvider.values.map((provider) {
                if (provider == EmailProvider.custom && !_useCustomSettings) {
                  return const SizedBox.shrink();
                }

                return ShadButton.outline(
                  onPressed: () => setState(() => _selectedProvider = provider),
                  backgroundColor: _selectedProvider == provider
                      ? ShadTheme.of(
                          context,
                        ).colorScheme.primary.withOpacity(0.1)
                      : null,
                  foregroundColor: _selectedProvider == provider
                      ? ShadTheme.of(context).colorScheme.primary
                      : null,
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(provider.displayName),
                      const SizedBox(width: 8),
                      SvgPicture.asset(
                        provider.iconPath,
                        width: 20,
                        height: 20,
                      ),
                    ],
                  ),
                );
              }).toList(),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCustomSettings() {
    return ShadCard(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'IMAP Settings (Incoming)',
              style: ShadTheme.of(
                context,
              ).textTheme.large.copyWith(fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 12),
            ShadInputFormField(
              controller: _customImapHostController,
              placeholder: const Text('imap.example.com'),
              validator: _validateRequired,
            ),
            const SizedBox(height: 8),
            Row(
              spacing: 16,
              children: [
                Expanded(
                  child: ShadInputFormField(
                    controller: _customImapPortController,
                    placeholder: const Text('993'),
                    keyboardType: TextInputType.number,
                    validator: _validatePort,
                  ),
                ),
                ShadSwitchFormField(
                  label: const Text('SSL'),
                  initialValue: _customImapSSL,
                  onChanged: (value) => setState(() => _customImapSSL = value),
                ),
              ],
            ),
            const SizedBox(height: 24),
            Text(
              'SMTP Settings (Outgoing)',
              style: ShadTheme.of(
                context,
              ).textTheme.large.copyWith(fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 12),
            ShadInputFormField(
              controller: _customSmtpHostController,
              placeholder: const Text('smtp.example.com'),
              validator: _validateRequired,
            ),
            const SizedBox(height: 8),
            Row(
              spacing: 16,
              children: [
                Expanded(
                  child: ShadInputFormField(
                    controller: _customSmtpPortController,
                    placeholder: const Text('587'),
                    keyboardType: TextInputType.number,
                    validator: _validatePort,
                  ),
                ),
                ShadSwitchFormField(
                  label: const Text('TLS'),
                  initialValue: _customSmtpTLS,
                  onChanged: (value) => setState(() => _customSmtpTLS = value),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  String? _validateEmail(String? value) {
    if (value == null || value.isEmpty) {
      return 'Please enter your email address';
    }
    if (!RegExp(r'^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$').hasMatch(value)) {
      return 'Please enter a valid email address';
    }
    return null;
  }

  String? _validatePassword(String? value) {
    if (value == null || value.isEmpty) {
      return 'Please enter your password';
    }
    if (value.length < 6) {
      return 'Password must be at least 6 characters';
    }
    return null;
  }

  String? _validateRequired(String? value) {
    if (value == null || value.isEmpty) {
      return 'This field is required';
    }
    return null;
  }

  String? _validatePort(String? value) {
    if (value == null || value.isEmpty) {
      return 'Please enter a port number';
    }
    final port = int.tryParse(value);
    if (port == null || port < 1 || port > 65535) {
      return 'Please enter a valid port number (1-65535)';
    }
    return null;
  }

  Future<void> _handleContinue() async {
    if (!_formKey.currentState!.validate()) {
      return;
    }

    final authProvider = Provider.of<AuthProvider>(context, listen: false);
    authProvider.clearError();

    ServerConfig? customImapConfig;
    ServerConfig? customSmtpConfig;

    if (_useCustomSettings) {
      customImapConfig = ServerConfig(
        host: _customImapHostController.text,
        port: int.parse(_customImapPortController.text),
        useSSL: _customImapSSL,
        useTLS: false,
        authMethod: AuthMethod.password,
      );

      customSmtpConfig = ServerConfig(
        host: _customSmtpHostController.text,
        port: int.parse(_customSmtpPortController.text),
        useSSL: false,
        useTLS: _customSmtpTLS,
        authMethod: AuthMethod.password,
      );
    }

    final success = await authProvider.authenticateWithPassword(
      email: _emailController.text,
      password: _passwordController.text,
      provider: _selectedProvider,
      customImapConfig: customImapConfig,
      customSmtpConfig: customSmtpConfig,
    );

    if (success && mounted) {
      // TODO: Navigate to main email screen
      Navigator.of(context).pop();
    }
  }
}
