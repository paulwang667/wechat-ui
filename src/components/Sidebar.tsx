import React from 'react';
import { 
  MessageSquare, 
  Users, 
  Database, 
  Activity, 
  Code2, 
  Gauge, 
  Settings,
  ShieldCheck,
  Zap
} from 'lucide-react';
import { Contact } from '../types/wechat';

export type TabType = 'chats' | 'contacts' | 'storage' | 'sync_diag' | 'rust_arch' | 'benchmarks';

interface SidebarProps {
  activeTab: TabType;
  onSelectTab: (tab: TabType) => void;
  currentUser: Contact;
  unreadTotal: number;
  outboxCount: number;
  onOpenProfile: () => void;
  isDarkMode: boolean;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeTab,
  onSelectTab,
  currentUser,
  unreadTotal,
  outboxCount,
  onOpenProfile,
  isDarkMode,
}) => {
  const navItems = [
    {
      id: 'chats' as TabType,
      label: 'Chats',
      icon: MessageSquare,
      badge: unreadTotal > 0 ? unreadTotal : null,
      badgeColor: 'bg-rose-500',
    },
    {
      id: 'contacts' as TabType,
      label: 'Contacts',
      icon: Users,
      badge: null,
    },
    {
      id: 'storage' as TabType,
      label: 'Storage & Compression',
      icon: Database,
      badge: null,
      highlight: true,
    },
    {
      id: 'sync_diag' as TabType,
      label: 'Sync Protocol Engine',
      icon: Activity,
      badge: outboxCount > 0 ? outboxCount : null,
      badgeColor: 'bg-amber-500',
    },
    {
      id: 'rust_arch' as TabType,
      label: 'Rust + Fenestra Blueprint',
      icon: Code2,
      badge: null,
    },
    {
      id: 'benchmarks' as TabType,
      label: 'Performance Benchmarks',
      icon: Gauge,
      badge: null,
    },
  ];

  return (
    <aside
      id="wechat_primary_sidebar"
      className={`w-16 flex flex-col items-center py-4 border-r select-none transition-colors ${
        isDarkMode 
          ? 'bg-[#18181c] border-zinc-800' 
          : 'bg-[#2e2e2e] border-zinc-700 text-zinc-300'
      }`}
    >
      {/* User Avatar with Profile click */}
      <button
        id="btn_user_profile"
        onClick={onOpenProfile}
        title={`${currentUser.name} (${currentUser.wechatId})`}
        className="relative group mb-6 focus:outline-none"
      >
        <img
          src={currentUser.avatar}
          alt={currentUser.name}
          className="w-10 h-10 rounded-lg object-cover ring-2 ring-emerald-500/60 group-hover:ring-emerald-400 transition-all shadow-md"
        />
        <span className="absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full bg-emerald-500 ring-2 ring-[#2e2e2e]" />
      </button>

      {/* Main Navigation Icons */}
      <nav className="flex-1 flex flex-col items-center space-y-4 w-full px-2">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = activeTab === item.id;

          return (
            <button
              key={item.id}
              id={`nav_btn_${item.id}`}
              onClick={() => onSelectTab(item.id)}
              title={item.label}
              className={`relative w-11 h-11 rounded-xl flex items-center justify-center transition-all ${
                isActive
                  ? 'bg-emerald-600 text-white shadow-lg shadow-emerald-900/30'
                  : 'text-zinc-400 hover:text-white hover:bg-white/10'
              }`}
            >
              <Icon className="w-5 h-5" />

              {/* Badge Counter */}
              {item.badge && (
                <span
                  className={`absolute -top-1 -right-1 min-w-[18px] h-[18px] px-1 text-[10px] font-bold text-white ${item.badgeColor || 'bg-rose-500'} rounded-full flex items-center justify-center border-2 border-[#2e2e2e] shadow`}
                >
                  {item.badge > 99 ? '99+' : item.badge}
                </span>
              )}

              {/* Active Indicator Bar on Left */}
              {isActive && (
                <span className="absolute -left-2 top-2.5 bottom-2.5 w-1 rounded-r-full bg-emerald-400" />
              )}
            </button>
          );
        })}
      </nav>

      {/* Bottom Info & Engine status */}
      <div className="flex flex-col items-center space-y-3 pt-2">
        <div 
          title="Rust Zero-Cost Memory Safety & Fenestra Native GPU Engine Active"
          className="w-9 h-9 rounded-lg bg-emerald-500/10 border border-emerald-500/30 flex items-center justify-center text-emerald-400"
        >
          <Zap className="w-4 h-4 text-emerald-400" />
        </div>

        <button
          id="btn_bottom_settings"
          onClick={onOpenProfile}
          title="Settings & System Diagnostics"
          className="w-9 h-9 rounded-lg text-zinc-400 hover:text-white hover:bg-white/10 flex items-center justify-center transition-colors"
        >
          <Settings className="w-4 h-4" />
        </button>
      </div>
    </aside>
  );
};
