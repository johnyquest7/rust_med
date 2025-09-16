<script lang="ts">
  import * as Sidebar from '$lib/components/ui/sidebar/index.js';
  import { House, Mic, FileText, User } from 'lucide-svelte';
  import { page } from '$app/state';
  import { getContext } from 'svelte';
  import type { AuthContext } from '$lib/types.js';

  // Get auth context
  const auth = getContext<AuthContext>('auth');

  // Handle case where auth context might not be available
  if (!auth) {
    console.error('Auth context not found in AppSidebar');
  }

  // Menu items for medical note generator
  const menuItems = [
    {
      title: 'Record Visit',
      url: '/',
      icon: Mic
    },
    {
      title: 'Medical Notes',
      url: '/notes',
      icon: FileText
    }
  ];

  function isActive(url: string): boolean {
    return page.url.pathname === url;
  }

  // Get user display information - using $derived for proper Svelte 5 reactivity
  let userDisplayName = $derived(auth?.state.user?.name || 'Not logged in');
  let userSpecialty = $derived(auth?.state.user?.specialty || 'Guest');
</script>

<Sidebar.Root collapsible="icon">
  <Sidebar.Header>
    <Sidebar.Menu class="mt-2">
      <div class="flex items-center gap-2">
        <div class="hidden aspect-square size-8 items-center justify-center rounded-lg md:flex">
          <Sidebar.Trigger class="hidden cursor-pointer justify-center px-2 md:flex" />
        </div>
        <div class="grid flex-1 px-2 text-left text-sm leading-tight md:px-0">
          <span class="truncate font-semibold">Medical Notes</span>
          <span class="truncate text-xs">AI-Powered Documentation</span>
        </div>
      </div>
    </Sidebar.Menu>
  </Sidebar.Header>

  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupLabel>Navigation</Sidebar.GroupLabel>
      <Sidebar.Menu>
        {#each menuItems as item}
          <Sidebar.MenuItem>
            <Sidebar.MenuButton isActive={isActive(item.url)}>
              {#snippet child({ props })}
                {@const IconComponent = item.icon}
                <a href={item.url} {...props}>
                  <IconComponent class="size-4" />
                  <span>{item.title}</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
        {/each}
      </Sidebar.Menu>
    </Sidebar.Group>

    <Sidebar.Separator />
    <!-- 
		<Sidebar.Group>
			<Sidebar.GroupLabel>Quick Actions</Sidebar.GroupLabel>
			<Sidebar.Menu>
				<Sidebar.MenuItem>
					<Sidebar.MenuButton>
						{#snippet child({ props })}
							<a href="/record" {...props}>
								<Mic class="size-4" />
								<span>Start Recording</span>
							</a>
						{/snippet}
					</Sidebar.MenuButton>
				</Sidebar.MenuItem>
				<Sidebar.MenuItem>
					<Sidebar.MenuButton>
						{#snippet child({ props })}
							<a href="/notes/new" {...props}>
								<FileText class="size-4" />
								<span>New Note</span>
							</a>
						{/snippet}
					</Sidebar.MenuButton>
				</Sidebar.MenuItem>
			</Sidebar.Menu>
		</Sidebar.Group> -->
  </Sidebar.Content>

  <Sidebar.Footer>
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <Sidebar.MenuButton size="lg">
          {#snippet child({ props })}
            <a href="/profile" {...props}>
              <div
                class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground"
              >
                <User class="size-4" />
              </div>
              <div class="grid flex-1 text-left text-sm leading-tight">
                <span class="truncate font-semibold">{userDisplayName}</span>
                <span class="truncate text-xs">{userSpecialty}</span>
              </div>
            </a>
          {/snippet}
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
  </Sidebar.Footer>
</Sidebar.Root>
