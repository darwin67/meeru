import 'package:flutter/material.dart' hide ThemeData;
import 'package:shadcn_ui/shadcn_ui.dart';
import 'package:provider/provider.dart';
import '../providers/auth_provider.dart';
import 'account_setup_screen.dart';
import 'account_list_screen.dart';

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
            spacing: 56,
            children: [
              // Logo/Icon
              Container(
                width: 120,
                height: 120,
                decoration: BoxDecoration(
                  color: ShadTheme.of(context).colorScheme.primary,
                  borderRadius: BorderRadius.circular(24),
                ),
                child: Icon(
                  LucideIcons.mail,
                  size: 64,
                  color: ShadTheme.of(context).colorScheme.primaryForeground,
                ),
              ),

              Center(
                child: SizedBox(
                  width: 480,
                  child: Column(
                    spacing: 18,
                    children: [
                      // Title
                      Text(
                        'Meeru',
                        style: ShadTheme.of(context).textTheme.h1,
                        textAlign: TextAlign.center,
                      ),
                      // Subtitle
                      Text(
                        'A modern, secure email client for all your accounts',
                        style: ShadTheme.of(context).textTheme.large.copyWith(
                              color: ShadTheme.of(context)
                                  .colorScheme
                                  .mutedForeground,
                            ),
                        textAlign: TextAlign.center,
                      ),
                    ],
                  ),
                ),
              ),

              // Features list
              const Center(
                child: SizedBox(
                  width: 480,
                  child: Column(
                    spacing: 24,
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      _FeatureItem(
                        icon: LucideIcons.lock,
                        title: 'Secure Storage',
                        description:
                            'Your credentials are encrypted and stored securely',
                      ),
                      _FeatureItem(
                        icon: LucideIcons.users,
                        title: 'Multi-Account Support',
                        description:
                            'Manage multiple email accounts in one place',
                      ),
                      _FeatureItem(
                        icon: LucideIcons.laptop,
                        title: 'Cross-Platform',
                        description: 'Works seamlessly across all your devices',
                      ),
                    ],
                  ),
                ),
              ),

              // Buttons
              Center(
                child: SizedBox(
                  width: 480,
                  child: Column(
                    spacing: 12,
                    children: [
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
                                    child: CircularProgressIndicator(
                                        strokeWidth: 2),
                                  )
                                : const Text('Get Started'),
                          );
                        },
                      ),

                      // Sign in link for existing users
                      ShadButton.ghost(
                        onPressed: () => _navigateToAccountList(context),
                        width: double.infinity,
                        child: const Text('Manage existing accounts'),
                      ),
                    ],
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _navigateToAccountSetup(BuildContext context) {
    Navigator.of(context).push(
      PageRouteBuilder(
        pageBuilder: (context, animation, secondaryAnimation) =>
            const AccountSetupScreen(),
        transitionsBuilder: (context, animation, secondaryAnimation, child) {
          return FadeTransition(opacity: animation, child: child);
        },
      ),
    );
  }

  void _navigateToAccountList(BuildContext context) {
    Navigator.of(context).pushReplacement(
      PageRouteBuilder(
        pageBuilder: (context, animation, secondaryAnimation) =>
            const AccountListScreen(),
        transitionsBuilder: (context, animation, secondaryAnimation, child) {
          return FadeTransition(opacity: animation, child: child);
        },
      ),
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
      spacing: 16,
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
        Expanded(
          child: Column(
            spacing: 4,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: ShadTheme.of(
                  context,
                ).textTheme.large.copyWith(fontWeight: FontWeight.w600),
              ),
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
