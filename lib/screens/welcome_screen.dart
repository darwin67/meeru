import 'package:flutter/material.dart' hide ThemeData;
import 'package:shadcn_ui/shadcn_ui.dart';
import 'package:provider/provider.dart';
import '../providers/auth_provider.dart';
import 'account_setup_screen.dart';

class WelcomeScreen extends StatelessWidget {
  const WelcomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: ShadTheme.of(context).colorScheme.background,
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            children: [
              const SizedBox(height: 60),

              // Logo/Icon
              Container(
                width: 120,
                height: 120,
                decoration: BoxDecoration(
                  color: ShadTheme.of(context).colorScheme.primary,
                  borderRadius: BorderRadius.circular(24),
                ),
                child: Icon(
                  Icons.email_outlined,
                  size: 64,
                  color: ShadTheme.of(context).colorScheme.primaryForeground,
                ),
              ),

              const SizedBox(height: 32),

              // Title
              Text(
                'Welcome to Meeru',
                style: ShadTheme.of(context).textTheme.h1,
                textAlign: TextAlign.center,
              ),

              const SizedBox(height: 16),

              // Subtitle
              Text(
                'A modern, secure email client for all your accounts',
                style: ShadTheme.of(context).textTheme.large.copyWith(
                  color: ShadTheme.of(context).colorScheme.mutedForeground,
                ),
                textAlign: TextAlign.center,
              ),

              const SizedBox(height: 48),

              // Features list
              Expanded(
                child: Column(
                  children: [
                    _FeatureItem(
                      icon: Icons.security_outlined,
                      title: 'Secure Storage',
                      description:
                          'Your credentials are encrypted and stored securely',
                    ),
                    const SizedBox(height: 24),
                    _FeatureItem(
                      icon: Icons.sync_outlined,
                      title: 'Multi-Account Support',
                      description:
                          'Manage multiple email accounts in one place',
                    ),
                    const SizedBox(height: 24),
                    _FeatureItem(
                      icon: Icons.devices_outlined,
                      title: 'Cross-Platform',
                      description: 'Works seamlessly across all your devices',
                    ),
                  ],
                ),
              ),

              const SizedBox(height: 48),

              // Get started button
              Consumer<AuthProvider>(
                builder: (context, authProvider, child) {
                  return ShadButton(
                    onPressed: authProvider.isLoading
                        ? null
                        : () => _navigateToAccountSetup(context),
                    width: double.infinity,
                    child: authProvider.isLoading
                        ? const SizedBox(
                            width: 16,
                            height: 16,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Text('Get Started'),
                  );
                },
              ),

              const SizedBox(height: 16),

              // Sign in link for existing users
              ShadButton.ghost(
                onPressed: () => _navigateToAccountSetup(context),
                child: const Text('I already have an account'),
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _navigateToAccountSetup(BuildContext context) {
    Navigator.of(context).pushReplacement(
      MaterialPageRoute(builder: (context) => const AccountSetupScreen()),
    );
  }
}

class _FeatureItem extends StatelessWidget {
  final IconData icon;
  final String title;
  final String description;

  const _FeatureItem({
    required this.icon,
    required this.title,
    required this.description,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Container(
          width: 48,
          height: 48,
          decoration: BoxDecoration(
            color: ShadTheme.of(context).colorScheme.muted,
            borderRadius: BorderRadius.circular(12),
          ),
          child: Icon(
            icon,
            size: 24,
            color: ShadTheme.of(context).colorScheme.mutedForeground,
          ),
        ),
        const SizedBox(width: 16),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: ShadTheme.of(
                  context,
                ).textTheme.large.copyWith(fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 4),
              Text(
                description,
                style: ShadTheme.of(context).textTheme.small.copyWith(
                  color: ShadTheme.of(context).colorScheme.mutedForeground,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
