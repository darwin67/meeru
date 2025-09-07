import 'package:flutter/material.dart' hide ThemeData;
import 'package:shadcn_ui/shadcn_ui.dart';
import 'package:provider/provider.dart';
import '../providers/auth_provider.dart';
import '../models/email_account.dart';
import 'account_setup_screen.dart';

class AccountListScreen extends StatefulWidget {
  const AccountListScreen({super.key});

  @override
  State<AccountListScreen> createState() => _AccountListScreenState();
}

class _AccountListScreenState extends State<AccountListScreen> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      Provider.of<AuthProvider>(context, listen: false).loadAccounts();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: ShadTheme.of(context).colorScheme.background,
      appBar: AppBar(
        title: const Text('Email Accounts'),
        backgroundColor: ShadTheme.of(context).colorScheme.background,
        foregroundColor: ShadTheme.of(context).colorScheme.foreground,
        elevation: 0,
        actions: [
          IconButton(
            onPressed: () => _addAccount(context),
            icon: const Icon(Icons.add),
            tooltip: 'Add Account',
          ),
        ],
      ),
      body: Consumer<AuthProvider>(
        builder: (context, authProvider, child) {
          if (authProvider.isLoading) {
            return const Center(child: CircularProgressIndicator());
          }

          if (authProvider.accounts.isEmpty) {
            return _buildEmptyState(context);
          }

          return _buildAccountList(context, authProvider.accounts);
        },
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () => _addAccount(context),
        backgroundColor: ShadTheme.of(context).colorScheme.primary,
        foregroundColor: ShadTheme.of(context).colorScheme.primaryForeground,
        child: const Icon(Icons.add),
      ),
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 80,
              height: 80,
              decoration: BoxDecoration(
                color: ShadTheme.of(context).colorScheme.muted,
                borderRadius: BorderRadius.circular(16),
              ),
              child: Icon(
                Icons.email_outlined,
                size: 40,
                color: ShadTheme.of(context).colorScheme.mutedForeground,
              ),
            ),
            const SizedBox(height: 24),
            Text(
              'No Email Accounts',
              style: ShadTheme.of(context).textTheme.h3,
            ),
            const SizedBox(height: 8),
            Text(
              'Add your first email account to get started',
              style: ShadTheme.of(context).textTheme.p.copyWith(
                color: ShadTheme.of(context).colorScheme.mutedForeground,
              ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 32),
            ShadButton(
              onPressed: () => _addAccount(context),
              child: const Text('Add Account'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildAccountList(BuildContext context, List<EmailAccount> accounts) {
    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: accounts.length,
      itemBuilder: (context, index) {
        final account = accounts[index];
        return _AccountCard(
          account: account,
          onDelete: () => _showDeleteConfirmation(context, account),
          onTap: () => _selectAccount(context, account),
        );
      },
    );
  }

  void _addAccount(BuildContext context) {
    Navigator.of(
      context,
    ).push(MaterialPageRoute(builder: (context) => const AccountSetupScreen()));
  }

  void _selectAccount(BuildContext context, EmailAccount account) {
    Provider.of<AuthProvider>(context, listen: false).switchAccount(account);
  }

  void _showDeleteConfirmation(BuildContext context, EmailAccount account) {
    showDialog(
      context: context,
      builder: (context) => ShadDialog(
        title: const Text('Delete Account'),
        description: Text(
          'Are you sure you want to delete ${account.email}? This action cannot be undone.',
        ),
        actions: [
          ShadButton.outline(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          ShadButton.destructive(
            onPressed: () {
              Navigator.of(context).pop();
              _deleteAccount(context, account);
            },
            child: const Text('Delete'),
          ),
        ],
      ),
    );
  }

  void _deleteAccount(BuildContext context, EmailAccount account) {
    Provider.of<AuthProvider>(context, listen: false).removeAccount(account.id);
  }
}

class _AccountCard extends StatelessWidget {
  final EmailAccount account;
  final VoidCallback onDelete;
  final VoidCallback onTap;

  const _AccountCard({
    required this.account,
    required this.onDelete,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Consumer<AuthProvider>(
      builder: (context, authProvider, child) {
        final isSelected = authProvider.currentAccount?.id == account.id;

        return Card(
          margin: const EdgeInsets.only(bottom: 12),
          child: InkWell(
            onTap: onTap,
            borderRadius: BorderRadius.circular(8),
            child: Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(8),
                border: isSelected
                    ? Border.all(
                        color: ShadTheme.of(context).colorScheme.primary,
                        width: 2,
                      )
                    : null,
                color: isSelected
                    ? ShadTheme.of(
                        context,
                      ).colorScheme.primary.withOpacity(0.05)
                    : null,
              ),
              child: Row(
                children: [
                  // Account avatar
                  CircleAvatar(
                    radius: 24,
                    backgroundColor: ShadTheme.of(context).colorScheme.primary,
                    child: Text(
                      account.email[0].toUpperCase(),
                      style: TextStyle(
                        color: ShadTheme.of(
                          context,
                        ).colorScheme.primaryForeground,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ),
                  const SizedBox(width: 16),

                  // Account details
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Expanded(
                              child: Text(
                                account.email,
                                style: ShadTheme.of(context).textTheme.p
                                    .copyWith(fontWeight: FontWeight.w600),
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                            if (isSelected)
                              Container(
                                padding: const EdgeInsets.symmetric(
                                  horizontal: 8,
                                  vertical: 2,
                                ),
                                decoration: BoxDecoration(
                                  color: ShadTheme.of(
                                    context,
                                  ).colorScheme.primary,
                                  borderRadius: BorderRadius.circular(12),
                                ),
                                child: Text(
                                  'Active',
                                  style: ShadTheme.of(context).textTheme.small
                                      .copyWith(
                                        color: ShadTheme.of(
                                          context,
                                        ).colorScheme.primaryForeground,
                                        fontSize: 10,
                                      ),
                                ),
                              ),
                          ],
                        ),
                        const SizedBox(height: 4),
                        Text(
                          account.provider.displayName,
                          style: ShadTheme.of(context).textTheme.small.copyWith(
                            color: ShadTheme.of(
                              context,
                            ).colorScheme.mutedForeground,
                          ),
                        ),
                        const SizedBox(height: 4),
                        Text(
                          '${account.imapConfig.host}:${account.imapConfig.port}',
                          style: ShadTheme.of(context).textTheme.small.copyWith(
                            color: ShadTheme.of(
                              context,
                            ).colorScheme.mutedForeground,
                          ),
                        ),
                      ],
                    ),
                  ),

                  // Delete button
                  IconButton(
                    onPressed: onDelete,
                    icon: Icon(
                      Icons.delete_outline,
                      color: ShadTheme.of(context).colorScheme.destructive,
                    ),
                    tooltip: 'Delete Account',
                  ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}
