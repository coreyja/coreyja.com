import axios from 'axios';
import { Thread, ThreadWithStitches, ThreadsListResponse } from '../types';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000/api';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

export const threadsApi = {
  listThreads: async (): Promise<Thread[]> => {
    const response = await api.get<ThreadsListResponse>('/threads');
    return response.data.threads;
  },

  getThread: async (id: string): Promise<ThreadWithStitches> => {
    const response = await api.get<ThreadWithStitches>(`/threads/${id}`);
    return response.data;
  },

  createThread: async (goal: string): Promise<Thread> => {
    const response = await api.post<Thread>('/threads', { goal });
    return response.data;
  },
};