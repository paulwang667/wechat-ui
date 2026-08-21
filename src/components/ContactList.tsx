import React, { useState } from 'react';
import { 
  Users, 
  Search, 
  MessageSquare, 
  Phone, 
  Video, 
  ShieldCheck, 
  MapPin, 
  UserCheck, 
  Bot,
  UserPlus
} from 'lucide-react';
import { Contact } from '../types/wechat';

interface ContactListProps {
  contacts: Contact[];
  onStartChatWithContact: (contact: Contact) => void;
  isDarkMode: boolean;
}

export const ContactList: React.FC<ContactListProps> = ({
  contacts,
  onStartChatWithContact,
  isDarkMode,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedContactId, setSelectedContactId] = useState<string>(contacts[0]?.id || '');

  const filteredContacts = contacts.filter((c) => {
    if (!searchQuery) return true;
    return (
      c.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      c.wechatId.toLowerCase().includes(searchQuery.toLowerCase()) ||
      c.bio.toLowerCase().includes(searchQuery.toLowerCase())
    );
  });

  const selectedContact = contacts.find((c) => c.id === selectedContactId) || contacts[0];

  return (
    <div
      id="wechat_contacts_panel"
      className="flex-1 flex h-full select-none overflow-hidden"
    >
      {/* Left Contacts List */}
      <div className={`w-72 border-r flex flex-col h-full transition-colors ${
        isDarkMode ? 'bg-[#25252b] border-zinc-800 text-zinc-200' : 'bg-[#ebecee] border-zinc-200 text-zinc-800'
      }`}>
        {/* Search Header */}
        <div className="p-3 border-b border-black/5 dark:border-white/5">
          <div className={`flex items-center px-2.5 py-1.5 rounded-md text-xs transition-colors ${
            isDarkMode ? 'bg-[#18181c] text-zinc-200' : 'bg-[#dcdcdc] text-zinc-800'
          }`}>
            <Search className="w-3.5 h-3.5 text-zinc-400 mr-2 shrink-0" />
            <input
              id="input_search_contacts"
              type="text"
              placeholder="Search contacts..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full bg-transparent border-none outline-none text-xs placeholder-zinc-400"
            />
          </div>
        </div>

        {/* Contacts directory */}
        <div className="flex-1 overflow-y-auto divide-y divide-black/5 dark:divide-white/5">
          {filteredContacts.map((contact) => {
            const isSelected = selectedContact?.id === contact.id;

            return (
              <div
                key={contact.id}
                id={`contact_item_${contact.id}`}
                onClick={() => setSelectedContactId(contact.id)}
                className={`px-3 py-2.5 flex items-center space-x-3 cursor-pointer transition-all ${
                  isSelected
                    ? isDarkMode ? 'bg-[#18181c]' : 'bg-[#c7c9cc]'
                    : isDarkMode ? 'hover:bg-[#2e2e36]' : 'hover:bg-[#e4e5e8]'
                }`}
              >
                <div className="relative shrink-0">
                  <img
                    src={contact.avatar}
                    alt={contact.name}
                    className="w-10 h-10 rounded-lg object-cover shadow-xs"
                  />
                  <span
                    className={`absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full ring-2 ring-white dark:ring-[#25252b] ${
                      contact.onlineStatus === 'online'
                        ? 'bg-emerald-500'
                        : contact.onlineStatus === 'busy'
                        ? 'bg-amber-500'
                        : 'bg-zinc-400'
                    }`}
                  />
                </div>

                <div className="flex-1 min-w-0">
                  <div className="flex items-center justify-between">
                    <h4 className="text-xs font-semibold text-zinc-900 dark:text-zinc-100 truncate">
                      {contact.name}
                    </h4>
                    {contact.category === 'bot' && (
                      <span className="text-[9px] px-1 py-0.2 rounded bg-emerald-500/10 text-emerald-500 font-mono font-bold">
                        BOT
                      </span>
                    )}
                  </div>
                  <p className="text-[10px] text-zinc-400 truncate mt-0.5 font-mono">
                    @{contact.wechatId}
                  </p>
                </div>
              </div>
            );
          })}
        </div>

        <div className="p-2 border-t border-black/5 dark:border-white/5 text-[10px] text-zinc-400 text-center font-mono">
          {filteredContacts.length} Contacts
        </div>
      </div>

      {/* Right Contact Details Card */}
      <div className={`flex-1 flex flex-col items-center justify-center p-8 transition-colors ${
        isDarkMode ? 'bg-[#1e1e24] text-zinc-100' : 'bg-[#f5f5f7] text-zinc-800'
      }`}>
        {selectedContact ? (
          <div className={`max-w-md w-full p-8 rounded-2xl border shadow-xl space-y-6 text-center ${
            isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
          }`}>
            {/* Avatar & Online state */}
            <div className="relative inline-block mx-auto">
              <img
                src={selectedContact.avatar}
                alt={selectedContact.name}
                className="w-24 h-24 rounded-2xl object-cover shadow-md mx-auto ring-4 ring-emerald-500/20"
              />
              <span
                className={`absolute bottom-0 right-0 w-5 h-5 rounded-full ring-4 ring-white dark:ring-[#25252b] flex items-center justify-center ${
                  selectedContact.onlineStatus === 'online'
                    ? 'bg-emerald-500'
                    : selectedContact.onlineStatus === 'busy'
                    ? 'bg-amber-500'
                    : 'bg-zinc-400'
                }`}
              />
            </div>

            {/* Names & IDs */}
            <div className="space-y-1">
              <h3 className="text-lg font-bold text-zinc-900 dark:text-zinc-50 flex items-center justify-center space-x-2">
                <span>{selectedContact.name}</span>
                {selectedContact.category === 'bot' && (
                  <Bot className="w-4 h-4 text-emerald-500" />
                )}
              </h3>
              <p className="text-xs text-zinc-400 font-mono">
                WeChat ID: <span className="text-zinc-700 dark:text-zinc-200 font-bold">{selectedContact.wechatId}</span>
              </p>
            </div>

            {/* Bio */}
            <p className="text-xs text-zinc-500 dark:text-zinc-400 leading-relaxed max-w-xs mx-auto">
              {selectedContact.bio}
            </p>

            {/* Region / Category metadata */}
            <div className="flex items-center justify-center space-x-4 text-xs text-zinc-400 font-mono pt-2 border-t border-black/5 dark:border-white/5">
              <span className="flex items-center space-x-1">
                <MapPin className="w-3.5 h-3.5 text-zinc-400" />
                <span>{selectedContact.region}</span>
              </span>
              <span>•</span>
              <span className="capitalize">{selectedContact.category}</span>
            </div>

            {/* Actions */}
            <div className="flex items-center justify-center space-x-3 pt-2">
              <button
                id="btn_send_message_to_contact"
                onClick={() => onStartChatWithContact(selectedContact)}
                className="px-6 py-2.5 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white font-medium text-xs flex items-center space-x-2 shadow-md transition-all cursor-pointer"
              >
                <MessageSquare className="w-4 h-4" />
                <span>Send Message</span>
              </button>

              <button
                onClick={() => alert(`Initiating audio call with ${selectedContact.name}...`)}
                title="Audio Call"
                className="p-2.5 rounded-xl border border-zinc-200 dark:border-zinc-700 hover:bg-black/5 dark:hover:bg-white/5 text-zinc-600 dark:text-zinc-300 transition-colors"
              >
                <Phone className="w-4 h-4" />
              </button>
            </div>
          </div>
        ) : (
          <div className="text-zinc-400 text-xs">Select a contact to view profile</div>
        )}
      </div>
    </div>
  );
};
